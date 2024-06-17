extern crate ffmpeg_next as ffmpeg;

use std::{mem::size_of, path::Path, sync::Arc, thread};

use eframe::egui::{
    ahash::HashMap,
    load::{ImageLoadResult, ImageLoader, ImagePoll, LoadError},
    mutex::Mutex,
    Color32, ColorImage, Context, SizeHint,
};

use ffmpeg::{
    format::{input, Pixel},
    media::Type,
};

use ffmpeg::software::scaling::flag::Flags;
use ffmpeg::util::frame::video::Video;

use crate::util::{image_to_egui_color_image, overlay_film_frame};

type Entry = Result<ImagePoll, String>;

/// egui image loader for video thumbnails
///
/// Uses ffmpeg to extract them
#[derive(Default)]
pub struct VideoThumbnailLoader {
    cache: Arc<Mutex<HashMap<String, Entry>>>,
}

impl VideoThumbnailLoader {
    pub const ID: &'static str = eframe::egui::generate_loader_id!(VideoThumbnailLoader);
}

const PROTOCOL: &str = "file://";

impl ImageLoader for VideoThumbnailLoader {
    fn id(&self) -> &str {
        Self::ID
    }

    fn load(&self, ctx: &Context, uri: &str, _: SizeHint) -> ImageLoadResult {
        let Some(path) = uri.strip_prefix(PROTOCOL).map(trim_extra_slash) else {
            return Err(LoadError::NotSupported);
        };

        let path = Path::new(path);

        if Path::new(uri)
            .extension()
            .and_then(|ext| ext.to_str())
            .map_or(false, |ext| ext != "mp4")
        {
            return Err(LoadError::NotSupported);
        }

        let mut cache = self.cache.lock();
        if let Some(entry) = cache.get(uri).cloned() {
            match entry {
                Ok(image) => {
                    Ok(image)
                }
                Err(err) => Err(LoadError::Loading(err)),
            }
        } else {
            let path = path.to_owned();
            cache.insert(uri.to_owned(), Ok(ImagePoll::Pending { size: None }));
            drop(cache);
            thread::Builder::new()
                .name(format!("thermal_cat::VideoThumbnailLoader::load({uri:?})"))
                .spawn({
                    let ctx = ctx.clone();
                    let cache = self.cache.clone();
                    let uri = uri.to_owned();
                    move || {
                        // extract the thing here
                        log::trace!("generating thumbnail {uri:?}");
                        log::trace!("path: {path:?}");
                        fn do_extraction(path: &Path) -> Result<Arc<ColorImage>, ffmpeg::Error> {
                            let mut ictx = input(path)?;
                            let input = ictx
                                .streams()
                                .best(Type::Video)
                                .ok_or(ffmpeg::Error::StreamNotFound)?;
                            let video_stream_index = input.index();
                            let context_decoder = ffmpeg::codec::context::Context::from_parameters(
                                input.parameters(),
                            )?;
                            let mut decoder = context_decoder.decoder().video()?;

                            let mut scaler = ffmpeg::software::scaling::Context::get(
                                decoder.format(),
                                decoder.width(),
                                decoder.height(),
                                Pixel::RGB24,
                                decoder.width(),
                                decoder.height(),
                                Flags::BILINEAR,
                            )?;
                            const MAX_FRAMES: usize = 10; // let's take the tenth frame at maximum.
                            let mut frame_idx = 0;
                            let mut rgb_frame = None;
                            for (stream, packet) in ictx.packets() {
                                if stream.index() == video_stream_index {
                                    log::trace!("decoding frame {frame_idx}");
                                    decoder.send_packet(&packet)?;
                                    let mut decoded = Video::empty();
                                    while decoder.receive_frame(&mut decoded).is_ok()
                                        && frame_idx < MAX_FRAMES
                                    {
                                        let mut scaled_frame = Video::empty();
                                        scaler.run(&decoded, &mut scaled_frame)?;
                                        rgb_frame = Some(scaled_frame);

                                        frame_idx += 1;
                                    }
                                }
                                if frame_idx >= MAX_FRAMES {
                                    break;
                                }
                            }
                            // Ok(Arc::new(ColorImage::new([0, 0], Color32::BLACK)))
                            rgb_frame
                                .map(|f| {
                                    // Arc::new(ColorImage::from_rgb(
                                    //     [f.width() as usize, f.height() as usize],
                                    //     f.data(0),
                                    // ))
                                    image::DynamicImage::ImageRgb8(
                                        image::RgbImage::from_raw(
                                            f.width(),
                                            f.height(),
                                            f.data(0).to_vec(),
                                        )
                                        .unwrap(),
                                    )
                                })
                                .map(overlay_film_frame)
                                .map(|img| {
                                    Arc::new(image_to_egui_color_image(
                                        image::DynamicImage::ImageRgb8(img),
                                    ))
                                })
                                .ok_or(ffmpeg::Error::StreamNotFound)
                        }

                        let result = match do_extraction(&path) {
                            Ok(image) => Ok(ImagePoll::Ready { image }),
                            Err(err) => Err(err.to_string()),
                        };

                        // debug print
                        match &result {
                            Ok(poll) => match poll {
                                ImagePoll::Ready { image } => {
                                    log::trace!("inserting image ready at uri: {}", uri);
                                }
                                _ => {
                                    log::trace!("inserting image not ready at uri: {}", uri);
                                }
                            },
                            Err(err) => {
                                log::error!("inserting error at uri: {}", uri);
                            }
                        }

                        cache.lock().insert(uri.clone(), result);
                        drop(cache);

                        ctx.request_repaint();
                        log::trace!("finished generating thumbnail {uri:?}");
                    }
                })
                .expect("failed to spawn thread");
            Ok(ImagePoll::Pending { size: None })
        }
    }

    fn forget(&self, uri: &str) {
        let _ = self.cache.lock().remove(uri);
    }

    fn forget_all(&self) {
        self.cache.lock().clear();
    }

    fn byte_size(&self) -> usize {
        self.cache
            .lock()
            .values()
            .map(|result| match result {
                Ok(poll) => match poll {
                    ImagePoll::Ready { image } => image.pixels.len() * size_of::<Color32>(),
                    _ => 0,
                },
                Err(err) => err.len(),
            })
            .sum()
    }
}

/// Remove the leading slash from the path if the target OS is Windows.
///
/// This is because Windows paths are not supposed to start with a slash.
/// For example, `file:///C:/path/to/file` is a valid URI, but `/C:/path/to/file` is not a valid path.
#[inline]
fn trim_extra_slash(s: &str) -> &str {
    if cfg!(target_os = "windows") {
        s.trim_start_matches('/')
    } else {
        s
    }
}
