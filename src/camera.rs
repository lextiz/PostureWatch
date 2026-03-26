use anyhow::{Context, Result};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
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
        if self.camera.is_none() {
            self.camera = Some(self.init_camera()?);
        }

        let cam = self.camera.as_mut().unwrap();
        let frame = match cam.frame() {
            Ok(f) => f,
            Err(_) => {
                self.camera = None;
                std::thread::sleep(std::time::Duration::from_millis(500));
                return self.capture_frame();
            }
        };

        let buffer = frame.buffer();
        let (w, h) = (frame.resolution().width(), frame.resolution().height());

        self.convert_to_jpeg(buffer, w, h)
    }

    fn convert_to_jpeg(&self, buffer: &[u8], w: u32, h: u32) -> Result<Vec<u8>> {
        let pixels = (w as usize) * (h as usize);

        // RGB (3 bytes/pixel)
        if buffer.len() == pixels * 3 {
            if let Some(img) = image::RgbImage::from_raw(w, h, buffer.to_vec()) {
                return self.encode_jpeg(image::DynamicImage::ImageRgb8(img));
            }
            // Try BGR -> RGB swap
            let mut swapped = buffer.to_vec();
            for chunk in swapped.chunks_mut(3) {
                chunk.swap(0, 2);
            }
            if let Some(img) = image::RgbImage::from_raw(w, h, swapped) {
                return self.encode_jpeg(image::DynamicImage::ImageRgb8(img));
            }
        }

        // RGBA (4 bytes/pixel)
        if buffer.len() == pixels * 4 {
            if let Some(rgba) = image::RgbaImage::from_raw(w, h, buffer.to_vec()) {
                let rgb = image::DynamicImage::ImageRgba8(rgba).to_rgb8();
                return self.encode_jpeg(image::DynamicImage::ImageRgb8(rgb));
            }
        }

        // YUYV (2 bytes/pixel)
        if buffer.len() == pixels * 2 {
            if let Some(rgb_data) = self.yuyv_to_rgb(buffer) {
                if let Some(img) = image::RgbImage::from_raw(w, h, rgb_data) {
                    return self.encode_jpeg(image::DynamicImage::ImageRgb8(img));
                }
            }
        }

        // YUV420 (1.5 bytes/pixel)
        if buffer.len() == pixels * 3 / 2 {
            if let Some(rgb_data) = self.yuv420_to_rgb(buffer, w as usize, h as usize) {
                if let Some(img) = image::RgbImage::from_raw(w, h, rgb_data) {
                    return self.encode_jpeg(image::DynamicImage::ImageRgb8(img));
                }
            }
        }

        // Try as encoded image
        if let Ok(img) = image::load_from_memory(buffer) {
            return self.encode_jpeg(img);
        }

        anyhow::bail!("Unknown format: {}x{} len={}", w, h, buffer.len())
    }

    fn encode_jpeg(&self, img: image::DynamicImage) -> Result<Vec<u8>> {
        let mut cursor = std::io::Cursor::new(Vec::new());
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, 80);
        img.write_with_encoder(encoder)
            .context("JPEG encode failed")?;
        Ok(cursor.into_inner())
    }

    fn yuyv_to_rgb(&self, buffer: &[u8]) -> Option<Vec<u8>> {
        let mut rgb = Vec::with_capacity(buffer.len() / 2 * 3);
        for chunk in buffer.chunks(4) {
            if chunk.len() != 4 {
                return None;
            }
            let (y0, u, y1, v) = (
                chunk[0] as f32,
                chunk[1] as f32 - 128.0,
                chunk[2] as f32,
                chunk[3] as f32 - 128.0,
            );
            for y in [y0, y1] {
                rgb.push((y + 1.402 * v).clamp(0.0, 255.0) as u8);
                rgb.push((y - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8);
                rgb.push((y + 1.772 * u).clamp(0.0, 255.0) as u8);
            }
        }
        Some(rgb)
    }

    fn yuv420_to_rgb(&self, buffer: &[u8], w: usize, h: usize) -> Option<Vec<u8>> {
        let y_len = w * h;
        let uv_w = w / 2;
        let uv_len = uv_w * (h / 2);
        if buffer.len() < y_len + 2 * uv_len {
            return None;
        }

        let (y_plane, uv_planes) = buffer.split_at(y_len);
        let (u_plane, v_plane) = uv_planes.split_at(uv_len);

        let mut rgb = Vec::with_capacity(y_len * 3);
        for row in 0..h {
            for col in 0..w {
                let y_val = y_plane[row * w + col] as f32;
                let uv_idx = (row / 2) * uv_w + (col / 2);
                let u = u_plane[uv_idx] as f32 - 128.0;
                let v = v_plane[uv_idx] as f32 - 128.0;
                rgb.push((y_val + 1.402 * v).clamp(0.0, 255.0) as u8);
                rgb.push((y_val - 0.344 * u - 0.714 * v).clamp(0.0, 255.0) as u8);
                rgb.push((y_val + 1.772 * u).clamp(0.0, 255.0) as u8);
            }
        }
        Some(rgb)
    }

    fn init_camera(&mut self) -> Result<Camera> {
        const MAX_CAMERAS: u32 = 5;

        if let Some(idx) = self.last_valid_index {
            if let Ok(cam) = self.try_init_camera(idx) {
                return Ok(cam);
            }
        }

        for i in 0..MAX_CAMERAS {
            let idx = (self.camera_index + i) % MAX_CAMERAS;
            if let Ok(cam) = self.try_init_camera(idx) {
                self.camera_index = idx;
                self.last_valid_index = Some(idx);
                return Ok(cam);
            }
        }

        anyhow::bail!("No cameras available")
    }

    fn try_init_camera(&self, index: u32) -> Result<Camera> {
        let requested =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        if let Ok(mut cam) = Camera::new(CameraIndex::Index(index), requested) {
            if cam.open_stream().is_ok() {
                return Ok(cam);
            }
        }

        let requested = RequestedFormat::new::<nokhwa::pixel_format::YuyvFormat>(
            RequestedFormatType::AbsoluteHighestFrameRate,
        );
        let mut cam =
            Camera::new(CameraIndex::Index(index), requested).context("Camera init failed")?;
        cam.open_stream().context("Stream open failed")?;
        Ok(cam)
    }
}
