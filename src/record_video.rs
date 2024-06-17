extern crate ffmpeg_next as ffmpeg;

use ffmpeg::ffi::av_log_set_level;
use ffmpeg::format::{output, output_as, Pixel};
use ffmpeg::rescale::TIME_BASE;
use ffmpeg::software::scaling::Flags;
use image::RgbImage;

use std::borrow::BorrowMut;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::thread;

use ffmpeg::{
    codec, decoder, encoder, format, frame, media, picture, Dictionary, Packet, Rational,
};

use crate::types::media_formats::VideoFormat;

#[derive(Debug, Clone)]
pub struct VideoRecordingSettings {
    pub output_path: PathBuf,
    pub format: VideoFormat,
    pub width: usize,
    pub height: usize,
    pub framerate: usize,
}

///
/// Starts a video recorsing session with the given settings.
/// Returns a channel to send video frames to. Closing the channel will stop the recording.
///
pub fn record_video(settings: VideoRecordingSettings) -> Result<Sender<RgbImage>, anyhow::Error> {
    unsafe {
        av_log_set_level(ffmpeg::ffi::AV_LOG_VERBOSE);
    }

    let (tx_frames, rx_frames) = channel();

    let mut octx = output_as(&settings.output_path, "mp4")
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

    encoder.set_height(settings.height as u32);
    encoder.set_width(settings.width as u32);
    encoder.set_frame_rate(Some(Rational::new(settings.framerate as i32, 1)));
    encoder.set_format(Pixel::YUV420P);

    encoder.set_qmin(10);
    encoder.set_qmax(51);
    encoder.set_bit_rate(1_000_000);
    encoder.set_me_range(16);
    encoder.set_i_quant_factor(0.71);

    encoder.set_time_base(Rational::new(1, settings.framerate as i32)); // todo change this?
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
            settings.width as u32,
            settings.height as u32,
            Pixel::YUV420P,
            settings.width as u32,
            settings.height as u32,
            Flags::BILINEAR,
        )
        .expect("failed to create scaler");

        let mut yuv_frame = frame::Video::new(
            Pixel::YUV420P,
            settings.width as u32,
            settings.height as u32,
        );
        let mut i = 0;
        while let Ok(frame) = rx_frames.recv() {
            let mut video_frame = convert_rgb_image_to_video_frame(frame);

            video_frame.set_kind(picture::Type::None);
            scaler.run(&video_frame, &mut yuv_frame).unwrap();
            yuv_frame.set_pts(Some((i as i64) * (1000_000 / settings.framerate as i64)));
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

    Ok(tx_frames)
}

pub fn convert_rgb_image_to_video_frame(img: RgbImage) -> frame::Video {
    let frame_width = img.width();
    let frame_height = img.height();

    let mut frm = frame::Video::new(Pixel::RGB24, frame_width as u32, frame_height as u32);

    let data_vec = img.as_raw();
    frm.data_mut(0).copy_from_slice(&data_vec);

    frm
}
