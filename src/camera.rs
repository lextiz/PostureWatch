use anyhow::{Context, Result};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{RequestedFormat, RequestedFormatType},
    Camera,
};

pub struct CameraState {
    camera: Option<Camera>,
    camera_index: u32,
    last_valid_index: Option<u32>,
}

impl CameraState {
    pub fn new() -> Self {
        Self {
            camera: None,
            camera_index: 0,
            last_valid_index: None,
        }
    }

    pub fn capture_frame(&mut self) -> Result<Vec<u8>> {
        // Try to reuse camera, but if it fails, reinitialize
        if self.camera.is_none() {
            match self.init_camera_with_retry() {
                Ok(cam) => self.camera = Some(cam),
                Err(e) => {
                    return Err(e).context("Failed to initialize any camera");
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
                    // Don't recurse immediately - try to find another camera
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    return self.capture_frame();
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

            // 3) YUYV/YUV422 (4 bytes for 2 pixels = 2 bytes per pixel)
            // Buffer size: w * h * 2
            // For 2560x1440: 2560*1440*2 = 7372800
            if buffer.len() == (w as usize) * (h as usize) * 2 {
                // Manual YUYV to RGB conversion
                // YUYV format: Y0 U0 Y1 V0 for every 2 pixels (shared U and V)
                let mut rgb_data = Vec::with_capacity((w as usize) * (h as usize) * 3);

                for chunk in buffer.chunks(4) {
                    if chunk.len() == 4 {
                        let y0 = chunk[0] as f32;
                        let y1 = chunk[2] as f32;
                        let u = chunk[1] as f32 - 128.0;
                        let v = chunk[3] as f32 - 128.0;

                        // First pixel
                        let r = (y0 + 1.402 * v).clamp(0.0, 255.0) as u8;
                        let g = (y0 - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8;
                        let b = (y0 + 1.772 * u).clamp(0.0, 255.0) as u8;
                        rgb_data.push(r);
                        rgb_data.push(g);
                        rgb_data.push(b);

                        // Second pixel
                        let r = (y1 + 1.402 * v).clamp(0.0, 255.0) as u8;
                        let g = (y1 - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8;
                        let b = (y1 + 1.772 * u).clamp(0.0, 255.0) as u8;
                        rgb_data.push(r);
                        rgb_data.push(g);
                        rgb_data.push(b);
                    }
                }

                if let Some(img) = image::RgbImage::from_raw(w, h, rgb_data) {
                    return encode_to_jpeg(image::DynamicImage::ImageRgb8(img));
                }
            }

            // 4) YUV420 (1.5 bytes per pixel: Y plane + U plane + V plane)
            // Buffer size: w * h * 3 / 2 (for planar format)
            // For 2560x1440: 2560*1440*1.5 = 5529600, which matches!
            if buffer.len() == (w as usize) * (h as usize) * 3 / 2 {
                // YUV420 planar format: Y plane followed by U and V
                // Y: w*h, U: w/2 * h/2, V: w/2 * h/2
                let y_plane_len = (w as usize) * (h as usize);
                let uv_width = w as usize / 2;
                let uv_height = h as usize / 2;
                let uv_plane_len = uv_width * uv_height;

                if buffer.len() >= y_plane_len + 2 * uv_plane_len {
                    let y_plane = &buffer[0..y_plane_len];
                    let u_plane = &buffer[y_plane_len..y_plane_len + uv_plane_len];
                    let v_plane =
                        &buffer[y_plane_len + uv_plane_len..y_plane_len + 2 * uv_plane_len];

                    let mut rgb_data = Vec::with_capacity(y_plane_len * 3);

                    for y in 0..h as usize {
                        for x in 0..w as usize {
                            let y_idx = y * w as usize + x;
                            let y_val = y_plane[y_idx] as f32;

                            // UV sampling is half in both directions
                            let uv_x = x / 2;
                            let uv_y = y / 2;
                            let uv_idx = uv_y * uv_width + uv_x;

                            let u = u_plane[uv_idx] as f32 - 128.0;
                            let v = v_plane[uv_idx] as f32 - 128.0;

                            // YUV to RGB conversion (BT.601)
                            let r = (y_val + 1.402 * v).clamp(0.0, 255.0) as u8;
                            let g = (y_val - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8;
                            let b = (y_val + 1.772 * u).clamp(0.0, 255.0) as u8;

                            rgb_data.push(r);
                            rgb_data.push(g);
                            rgb_data.push(b);
                        }
                    }

                    if let Some(img) = image::RgbImage::from_raw(w, h, rgb_data) {
                        return encode_to_jpeg(image::DynamicImage::ImageRgb8(img));
                    }
                }
            }

            // 5) Last resort: try to interpret buffer as an encoded image (jpeg/png) in-memory
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

    fn init_camera_with_retry(&mut self) -> Result<Camera> {
        // Try multiple camera indices to find one that works
        let max_cameras = 5;

        // First try the last known valid index
        if let Some(last_idx) = self.last_valid_index {
            if let Ok(cam) = self.try_init_camera(last_idx) {
                self.camera_index = last_idx;
                return Ok(cam);
            }
        }

        // Try cameras from current index onwards
        for i in 0..max_cameras {
            let idx = (self.camera_index + i) % max_cameras;
            match self.try_init_camera(idx) {
                Ok(cam) => {
                    self.camera_index = idx;
                    self.last_valid_index = Some(idx);
                    println!("Using camera index {}", idx);
                    return Ok(cam);
                }
                Err(e) => {
                    eprintln!("Camera {} not available: {:?}", idx, e);
                }
            }
        }

        anyhow::bail!("No available cameras found")
    }

    fn try_init_camera(&self, index: u32) -> Result<Camera> {
        // Try RGB format first (most common for webcams)
        let requested =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        if let Ok(mut cam) = Camera::new(nokhwa::utils::CameraIndex::Index(index), requested) {
            if cam.open_stream().is_ok() {
                return Ok(cam);
            }
        }

        // Fallback: try YUYV format
        let requested = RequestedFormat::new::<nokhwa::pixel_format::YuyvFormat>(
            RequestedFormatType::AbsoluteHighestFrameRate,
        );
        let mut cam = Camera::new(nokhwa::utils::CameraIndex::Index(index), requested)
            .context("Failed to create camera")?;
        cam.open_stream().context("Failed to open camera stream")?;
        Ok(cam)
    }
}
