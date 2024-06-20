extern crate ffmpeg_next as ffmpeg;
use std::sync::mpsc::Sender;
use std::sync::Mutex;
use std::{path::PathBuf, sync::mpsc::channel, thread};

use crate::util::rgba8_to_rgb8;
use crate::{types::media_formats::VideoFormat, util::pathify_string};
use anyhow::anyhow;
use chrono::{DateTime, Local};
use ffmpeg::ffi::av_log_set_level;
use ffmpeg::format::{output_as, Pixel};
use ffmpeg::rescale::TIME_BASE;
use ffmpeg::software::scaling::Flags;
use ffmpeg::{codec, encoder, format, frame, picture, Dictionary, Packet, Rational};
use image::RgbImage;
use std::borrow::BorrowMut;

use super::recorder::{Recorder, RecorderState};

pub struct VideoRecorder {
    // Params
    destination_folder: PathBuf,
    name_prefix: String,
    video_format: VideoFormat,

    // State
    tx_frames: Option<Sender<RgbImage>>,

    // Output info
    output_file: Option<PathBuf>,
    curr_state: RecorderState,
}

impl VideoRecorder {
    pub fn new(
        destination_folder: PathBuf,
        name_prefix: String,
        video_format: VideoFormat,
    ) -> VideoRecorder {
        VideoRecorder {
            destination_folder,
            name_prefix,
            video_format,
            tx_frames: None,
            output_file: None,
            curr_state: RecorderState::Initial,
        }
    }
}

impl Recorder for VideoRecorder {
    fn start(
        &mut self,
        params: super::recorder::RecorderStreamParams,
    ) -> Result<(), anyhow::Error> {
        unsafe {
            av_log_set_level(ffmpeg::ffi::AV_LOG_VERBOSE);
        }

        std::fs::create_dir_all(self.destination_folder.clone())?;
        let current_local: DateTime<Local> = Local::now();

        let filename = format!(
            "{}_{}.{}",
            pathify_string(self.name_prefix.clone()),
            current_local.format("%Y-%m-%d_%H-%M-%S"),
            self.video_format.extension()
        );

        let (tx_frames, rx_frames) = channel();
        self.tx_frames = Some(tx_frames);
        let full_path = self.destination_folder.join(filename.clone());
        self.output_file = Some(full_path.clone());
        let mut octx = output_as(&full_path, "mp4")
            .map_err(|err| anyhow::anyhow!("failed to create output: {}", err))?;

        let global_header = octx
            .format()
            .flags()
            .contains(format::flag::Flags::GLOBAL_HEADER);

        let the_codec = encoder::find(codec::Id::H264);
        let mut ost = octx.add_stream(the_codec)?;
        ost.set_time_base(TIME_BASE);
        let ost_index = ost.index(); // output stream
        let mut encoder = codec::context::Context::from_parameters(ost.parameters())?
            .encoder()
            .video()?;

        encoder.set_height(params.height as u32);
        encoder.set_width(params.width as u32);
        encoder.set_frame_rate(Some(Rational::new(params.framerate as i32, 1)));
        encoder.set_format(Pixel::YUV420P);

        encoder.set_qmin(10);
        encoder.set_qmax(51);
        encoder.set_bit_rate(1_000_000);
        encoder.set_me_range(16);
        encoder.set_i_quant_factor(0.71);

        encoder.set_time_base(Rational::new(1, params.framerate as i32)); // todo change this?
        if global_header {
            encoder.set_flags(ffmpeg::codec::flag::Flags::GLOBAL_HEADER);
        }
        let mut x264_opts = Dictionary::new();

        x264_opts.set("preset", "medium");

        let mut encoder = encoder
            .open_as_with(the_codec, x264_opts)
            .expect("error opening libx264 encoder with supplied settings");

        ost.set_parameters(&encoder);

        octx.write_header()?;
        let mutexed_octx = Mutex::new(octx);

        thread::spawn(move || {
            let mut scaler = ffmpeg::software::scaling::context::Context::get(
                Pixel::RGB24,
                params.width as u32,
                params.height as u32,
                Pixel::YUV420P,
                params.width as u32,
                params.height as u32,
                Flags::BILINEAR,
            )
            .expect("failed to create scaler");

            let mut yuv_frame =
                frame::Video::new(Pixel::YUV420P, params.width as u32, params.height as u32);
            let mut i = 0;
            while let Ok(frame) = rx_frames.recv() {
                let mut video_frame = convert_rgb_image_to_video_frame(frame);

                video_frame.set_kind(picture::Type::None);
                scaler.run(&video_frame, &mut yuv_frame).unwrap();
                yuv_frame.set_pts(Some((i as i64) * (1_000_000 / params.framerate as i64)));
                match encoder.send_frame(&yuv_frame) {
                    Ok(_) => {}
                    Err(err) => {
                        log::error!("failed to send frame: {}", err);
                        break;
                    }
                }

                let mut encoded = Packet::empty();
                while encoder.receive_packet(&mut encoded).is_ok() {
                    encoded.set_stream(ost_index);
                    // encoded.rescale_ts(self.decoder.time_base(), ost_time_base);
                    let mut octx = mutexed_octx.lock().unwrap();
                    encoded.write_interleaved(octx.borrow_mut()).unwrap();
                }
                i += 1;
            }

            let _ = encoder.send_eof().inspect_err(|err| {
                log::error!("failed to finish encoding: {}", err);
            });

            if let Err(err) = mutexed_octx.lock().unwrap().write_trailer() {
                log::error!("failed to write trailer: {}", err);
            }
        });
        self.curr_state = RecorderState::Recording;
        Ok(())
    }

    fn process_result(
        &mut self,
        result: &crate::thermal_capturer::ThermalCapturerResult,
    ) -> Result<(), anyhow::Error> {
        if let Some(tx_frames) = &self.tx_frames {
            let rgb_img = rgba8_to_rgb8(
                image::RgbaImage::from_raw(
                    result.image.width() as u32,
                    result.image.height() as u32,
                    result.image.as_raw().into(),
                )
                .ok_or(anyhow!("Failed to create image when copying frame"))?,
            );

            tx_frames.send(rgb_img)?;
        }
        Ok(())
    }

    fn state(&self) -> RecorderState {
        self.curr_state
    }

    fn files_created(&self) -> Vec<PathBuf> {
        self.output_file.clone().into_iter().collect()
    }

    fn stop(&mut self) -> Result<(), anyhow::Error> {
        self.curr_state = RecorderState::Done;
        self.tx_frames = None; // Drop the sender

        Ok(())
    }
    fn is_continuous(&self) -> bool {
        true
    }
}

pub fn convert_rgb_image_to_video_frame(img: RgbImage) -> frame::Video {
    let frame_width = img.width();
    let frame_height = img.height();

    let mut frm = frame::Video::new(Pixel::RGB24, frame_width, frame_height);

    let data_vec = img.as_raw();
    frm.data_mut(0).copy_from_slice(data_vec);

    frm
}
