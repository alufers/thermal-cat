use anyhow::Error;
use chrono::{DateTime, Local};
use image::RgbImage;
use once_cell::race::OnceBool;
use video_rs::ffmpeg::format::Pixel;

use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};
use std::thread;
use video_rs::encode::Settings;
use video_rs::ffmpeg::ffi::{av_image_copy, av_image_fill_arrays, AVPixelFormat};
use video_rs::ffmpeg::frame::Video as Frame;
use video_rs::time::Time;
use video_rs::Encoder;

use crate::types::media_formats::VideoFormat;

#[derive(Debug, Clone)]
pub struct VideoRecordingSettings {
    pub output_path: PathBuf,
    pub format: VideoFormat,
    pub width: usize,
    pub height: usize,
    pub framerate: usize,
}

static DID_INIT_VIDEO_RS: OnceBool = OnceBool::new();

///
/// Starts a video recorsing session with the given settings.
/// Returns a channel to send video frames to. Closing the channel will stop the recording.
///
pub fn record_video(settings: VideoRecordingSettings) -> Result<Sender<RgbImage>, anyhow::Error> {
    DID_INIT_VIDEO_RS.get_or_try_init(|| {
        video_rs::init().map_err(|err| anyhow::anyhow!("{}", err))?;
        Ok::<bool, anyhow::Error>(true)
    })?;

    let (tx_frames, rx_frames) = channel();

    let current_local: DateTime<Local> = Local::now();

    let preset = Settings::preset_h264_yuv420p(settings.width, settings.height, false);

    let mut encoder = Encoder::new(settings.output_path, preset)
        .map_err(|err| anyhow::anyhow!("failed to create encoder: {}", err))?;

    let duration: Time = Time::from_nth_of_a_second(settings.framerate);
    let mut position = Time::zero();

    thread::spawn(move || {
        while let Ok(frame) = rx_frames.recv() {
            let video_frame = convert_rgb_image_to_video_frame(frame)
                .map_err(|err| anyhow::anyhow!("failed to convert frame: {}", err));

            match video_frame {
                Err(err) => {
                    log::error!("failed to convert frame: {}", err);
                    break;
                }
                Ok(video_frame) => {
                    let mut video_frame = video_frame;
                    video_frame.set_pts(position.with_time_base(encoder.time_base()).into_value());
                    let res = encoder.encode_raw(video_frame).inspect_err(|err| {
                        log::error!("failed to encode frame: {}", err);
                    });
                    if res.is_err() {
                        break;
                    }
                }
            }
            position = position.aligned_with(&duration).add();
        }

        let _ = encoder.finish().inspect_err(|err| {
            log::error!("failed to finish encoding: {}", err);
        });
    });

    Ok(tx_frames)
}

pub fn convert_rgb_image_to_video_frame(img: RgbImage) -> Result<Frame, Error> {
    unsafe {
        let frame_width = img.width();
        let frame_height = img.height();

        // Temporary frame structure to place correctly formatted data and linesize stuff in, which
        // we'll copy later.
        let mut frame_tmp = Frame::empty();
        let frame_tmp_ptr = frame_tmp.as_mut_ptr();

        // This does not copy the data, but it sets the `frame_tmp` data and linesize pointers
        // correctly.
        let bytes_copied = av_image_fill_arrays(
            (*frame_tmp_ptr).data.as_ptr() as *mut *mut u8,
            (*frame_tmp_ptr).linesize.as_ptr() as *mut i32,
            img.as_raw().as_ptr(),
            AVPixelFormat::AV_PIX_FMT_RGB24,
            frame_width as i32,
            frame_height as i32,
            1,
        );

        if bytes_copied != img.as_raw().len() as i32 {
            return Err(anyhow::anyhow!(
                "failed to copy image data to frame: {} != {}",
                bytes_copied,
                img.as_raw().len()
            ));
        }

        let mut frame = Frame::new(Pixel::RGB24, frame_width, frame_height);
        let frame_ptr = frame.as_mut_ptr();

        // Do the actual copying.
        av_image_copy(
            (*frame_ptr).data.as_ptr() as *mut *mut u8,
            (*frame_ptr).linesize.as_ptr() as *mut i32,
            (*frame_tmp_ptr).data.as_ptr() as *mut *const u8,
            (*frame_tmp_ptr).linesize.as_ptr(),
            AVPixelFormat::AV_PIX_FMT_RGB24,
            frame_width as i32,
            frame_height as i32,
        );

        Ok(frame)
    }
}
