use anyhow::{Context, Result};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{RequestedFormat, RequestedFormatType},
    Camera,
};

pub struct CameraState {
    camera: Option<Camera>,
}

impl CameraState {
    pub fn new() -> Self {
        Self { camera: None }
    }

    pub fn capture_frame(&mut self) -> Result<Vec<u8>> {
        // Try to reuse camera, but if it fails, reinitialize
        if self.camera.is_none() {
            match self.init_camera() {
                Ok(cam) => self.camera = Some(cam),
                Err(e) => {
                    eprintln!("Camera init error: {:?}", e);
                    // Wait a bit and retry
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    match self.init_camera() {
                        Ok(cam) => self.camera = Some(cam),
                        Err(e) => return Err(e).context("Failed to initialize camera after retry"),
                    }
                }
            }
        }

        if let Some(cam) = &mut self.camera {
            // Try to capture, if fails reinitialize
            let frame_result = cam.frame();

            let frame = match frame_result {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Camera frame capture error: {:?}, reinitializing...", e);
                    self.camera = None;
                    return self.capture_frame(); // Recursive retry
                }
            };
            let buffer = frame.buffer();
            let (w, h) = (frame.resolution().width(), frame.resolution().height());

            // Try to handle common pixel layouts robustly:
            // - RGB (3 bytes/pixel)
            // - RGBA (4 bytes/pixel)
            // - BGR (3 bytes/pixel, channel order swapped)
            // Fallback: return a helpful error with diagnostics.

            // Helper to encode DynamicImage to JPEG bytes
            fn encode_to_jpeg(img: image::DynamicImage) -> anyhow::Result<Vec<u8>> {
                let mut cursor = std::io::Cursor::new(Vec::new());
                img.write_to(&mut cursor, image::ImageOutputFormat::Jpeg(80))
                    .context("Failed to encode as JPEG")?;
                Ok(cursor.into_inner())
            }

            // 1) Exact RGB (3 bytes per pixel)
            if buffer.len() == (w as usize) * (h as usize) * 3 {
                if let Some(img) = image::RgbImage::from_raw(w, h, buffer.to_vec()) {
                    return encode_to_jpeg(image::DynamicImage::ImageRgb8(img));
                }
                // If from_raw failed despite matching size, try BGR -> RGB swap
                let mut swapped = buffer.to_vec();
                for chunk in swapped.chunks_mut(3) {
                    if chunk.len() == 3 {
                        chunk.swap(0, 2);
                    }
                }
                if let Some(img) = image::RgbImage::from_raw(w, h, swapped) {
                    return encode_to_jpeg(image::DynamicImage::ImageRgb8(img));
                }
            }

            // 2) RGBA (4 bytes per pixel) -> convert to RGB
            if buffer.len() == (w as usize) * (h as usize) * 4 {
                if let Some(rgba) = image::RgbaImage::from_raw(w, h, buffer.to_vec()) {
                    let rgb = image::DynamicImage::ImageRgba8(rgba).to_rgb8();
                    return encode_to_jpeg(image::DynamicImage::ImageRgb8(rgb));
                }
            }

            // 3) Last resort: try to interpret buffer as an encoded image (jpeg/png) in-memory
            if let Ok(img) = image::load_from_memory(buffer) {
                return encode_to_jpeg(img);
            }

            // Diagnostic error to help debugging on unknown formats
            anyhow::bail!(
                "Failed to parse raw buffer as RGB image. resolution={}x{} buffer_len={}",
                w,
                h,
                buffer.len()
            );
        }

        anyhow::bail!("Camera is not initialized")
    }

    fn init_camera(&self) -> Result<Camera> {
        let index = nokhwa::utils::CameraIndex::Index(0);
        let requested =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        let mut cam = Camera::new(index, requested).context("Failed to create camera")?;
        cam.open_stream().context("Failed to open camera stream")?;
        Ok(cam)
    }
}
