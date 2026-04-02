use crate::log_info;
use anyhow::{Context, Result};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use std::collections::HashSet;

pub struct CameraState {
    camera: Option<Camera>,
    camera_index: u32,
    last_valid_index: Option<u32>,
    current_index: Option<u32>,
    skipped_indexes: HashSet<u32>,
    black_frame_streak: u32,
}

impl CameraState {
    pub fn new() -> Self {
        Self {
            camera: None,
            camera_index: 0,
            last_valid_index: None,
            current_index: None,
            skipped_indexes: HashSet::new(),
            black_frame_streak: 0,
        }
    }

    pub fn capture_frame(&mut self) -> Result<Vec<u8>> {
        loop {
            if self.camera.is_none() {
                self.camera = Some(self.init_camera()?);
            }

            let Some(cam) = self.camera.as_mut() else {
                continue;
            };

            match cam.frame() {
                Ok(frame) => {
                    let buffer = frame.buffer();
                    let (w, h) = (frame.resolution().width(), frame.resolution().height());
                    if self.is_mostly_black_frame(buffer, w, h) {
                        self.black_frame_streak += 1;
                        if self.black_frame_streak >= 3 {
                            self.rotate_from_current_camera("camera returned only black frames");
                        }
                        std::thread::sleep(std::time::Duration::from_millis(200));
                        continue;
                    }
                    self.black_frame_streak = 0;
                    return self.convert_to_jpeg(buffer, w, h);
                }
                Err(_) => {
                    self.rotate_from_current_camera("camera frame capture failed");
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        }
    }

    pub fn shutdown(&mut self) {
        if let Some(mut cam) = self.camera.take() {
            let _ = cam.stop_stream();
        }
        self.current_index = None;
        self.black_frame_streak = 0;
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
        const MAX_CAMERAS: u32 = 12;

        if self.skipped_indexes.len() as u32 >= MAX_CAMERAS {
            self.skipped_indexes.clear();
        }

        if let Some(idx) = self
            .last_valid_index
            .filter(|idx| !self.skipped_indexes.contains(idx))
        {
            if let Ok(cam) = self.try_init_camera(idx) {
                self.current_index = Some(idx);
                return Ok(cam);
            }
        }

        for i in 0..MAX_CAMERAS {
            let idx = (self.camera_index + i) % MAX_CAMERAS;
            if self.skipped_indexes.contains(&idx) {
                continue;
            }
            if let Ok(cam) = self.try_init_camera(idx) {
                self.camera_index = idx;
                self.last_valid_index = Some(idx);
                self.current_index = Some(idx);
                return Ok(cam);
            }
        }

        self.skipped_indexes.clear();
        anyhow::bail!("No cameras available")
    }

    fn rotate_from_current_camera(&mut self, reason: &str) {
        if let Some(idx) = self.current_index {
            self.skipped_indexes.insert(idx);
            self.camera_index = idx.saturating_add(1);
            log_info!("Switching camera from index {}: {}", idx, reason);
        }
        self.camera = None;
        self.current_index = None;
        self.black_frame_streak = 0;
    }

    fn is_mostly_black_frame(&self, buffer: &[u8], w: u32, h: u32) -> bool {
        let pixels = (w as usize) * (h as usize);
        if pixels == 0 {
            return true;
        }

        let stride = if buffer.len() == pixels * 3 {
            3
        } else if buffer.len() == pixels * 4 {
            4
        } else if buffer.len() == pixels * 2 {
            2
        } else {
            return false;
        };

        let mut dark = 0usize;
        let mut sampled = 0usize;

        for chunk in buffer.chunks(stride).step_by(10) {
            if chunk.len() < stride {
                continue;
            }
            sampled += 1;
            let luma = if stride == 2 {
                u32::from(chunk[0])
            } else {
                (u32::from(chunk[0]) + u32::from(chunk[1]) + u32::from(chunk[2])) / 3
            };
            if luma <= 8 {
                dark += 1;
            }
        }

        sampled > 0 && (dark * 100 / sampled) >= 98
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yuyv_to_rgb_converts_two_pixels() {
        let state = CameraState::new();
        let yuyv = [100_u8, 128_u8, 150_u8, 128_u8];
        let rgb = state
            .yuyv_to_rgb(&yuyv)
            .expect("yuyv conversion should succeed");
        assert_eq!(rgb.len(), 6);
        assert_eq!(rgb[0], 100);
        assert_eq!(rgb[1], 100);
        assert_eq!(rgb[2], 100);
        assert_eq!(rgb[3], 150);
        assert_eq!(rgb[4], 150);
        assert_eq!(rgb[5], 150);
    }

    #[test]
    fn yuyv_to_rgb_rejects_partial_chunks() {
        let state = CameraState::new();
        assert!(state.yuyv_to_rgb(&[1, 2, 3]).is_none());
    }

    #[test]
    fn shutdown_without_camera_is_safe() {
        let mut state = CameraState::new();
        state.shutdown();
    }

    #[test]
    fn yuv420_to_rgb_converts_minimal_frame() {
        let state = CameraState::new();
        let y_plane = [100_u8, 100_u8, 100_u8, 100_u8];
        let u_plane = [128_u8];
        let v_plane = [128_u8];
        let mut input = Vec::new();
        input.extend_from_slice(&y_plane);
        input.extend_from_slice(&u_plane);
        input.extend_from_slice(&v_plane);

        let rgb = state
            .yuv420_to_rgb(&input, 2, 2)
            .expect("yuv420 conversion should succeed");
        assert_eq!(
            rgb,
            vec![100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100]
        );
    }

    #[test]
    fn yuv420_to_rgb_rejects_short_input() {
        let state = CameraState::new();
        assert!(state.yuv420_to_rgb(&[0, 0, 0], 2, 2).is_none());
    }

    #[test]
    fn convert_to_jpeg_handles_rgb_and_errors_on_unknown() {
        let state = CameraState::new();
        let rgb = vec![255_u8, 0_u8, 0_u8, 0_u8, 255_u8, 0_u8];
        let jpeg = state
            .convert_to_jpeg(&rgb, 2, 1)
            .expect("rgb should convert to jpeg");
        assert!(!jpeg.is_empty());
        assert_eq!(&jpeg[0..2], &[0xFF, 0xD8]);

        let err = state
            .convert_to_jpeg(&[1, 2, 3, 4, 5], 2, 2)
            .expect_err("unknown buffer format should fail");
        assert!(err.to_string().contains("Unknown format"));
    }

    #[test]
    fn convert_to_jpeg_handles_rgba_yuyv_yuv420_and_encoded_input() {
        let state = CameraState::new();

        let rgba = vec![255_u8, 0_u8, 0_u8, 255_u8, 0_u8, 255_u8, 0_u8, 255_u8];
        let rgba_jpeg = state
            .convert_to_jpeg(&rgba, 2, 1)
            .expect("rgba should convert to jpeg");
        assert_eq!(&rgba_jpeg[0..2], &[0xFF, 0xD8]);

        let yuyv = vec![100_u8, 128_u8, 150_u8, 128_u8];
        let yuyv_jpeg = state
            .convert_to_jpeg(&yuyv, 2, 1)
            .expect("yuyv should convert to jpeg");
        assert_eq!(&yuyv_jpeg[0..2], &[0xFF, 0xD8]);

        let mut yuv420 = vec![100_u8, 100_u8, 100_u8, 100_u8];
        yuv420.extend_from_slice(&[128_u8]);
        yuv420.extend_from_slice(&[128_u8]);
        let yuv420_jpeg = state
            .convert_to_jpeg(&yuv420, 2, 2)
            .expect("yuv420 should convert to jpeg");
        assert_eq!(&yuv420_jpeg[0..2], &[0xFF, 0xD8]);

        let png = vec![
            137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,
            8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 156, 99, 248, 207,
            192, 240, 31, 0, 5, 0, 1, 255, 137, 153, 61, 29, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66,
            96, 130,
        ];
        let encoded_jpeg = state
            .convert_to_jpeg(&png, 1, 1)
            .expect("encoded image input should convert to jpeg");
        assert_eq!(&encoded_jpeg[0..2], &[0xFF, 0xD8]);
    }

    #[test]
    fn detects_mostly_black_frames_across_raw_formats() {
        let state = CameraState::new();
        let rgb_black = vec![0_u8; 3 * 100];
        let rgba_black = vec![0_u8; 4 * 100];
        let yuyv_black = vec![0_u8; 2 * 100];
        let rgb_non_black = vec![64_u8; 3 * 100];

        assert!(state.is_mostly_black_frame(&rgb_black, 10, 10));
        assert!(state.is_mostly_black_frame(&rgba_black, 10, 10));
        assert!(state.is_mostly_black_frame(&yuyv_black, 10, 10));
        assert!(!state.is_mostly_black_frame(&rgb_non_black, 10, 10));
    }

    #[test]
    fn rotate_marks_current_camera_for_retry_later() {
        let mut state = CameraState::new();
        state.current_index = Some(2);

        state.rotate_from_current_camera("test");

        assert!(state.camera.is_none());
        assert!(state.current_index.is_none());
        assert!(state.skipped_indexes.contains(&2));
        assert_eq!(state.camera_index, 3);
    }
}
