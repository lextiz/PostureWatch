use anyhow::{Context, Result};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraFormat, FrameFormat, Resolution, RequestedFormat, RequestedFormatType},
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
        if self.camera.is_none() {
            let index = nokhwa::utils::CameraIndex::Index(0);
            let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
            let mut cam = Camera::new(index, requested).context("Failed to initialize camera")?;
            cam.open_stream().context("Failed to open camera stream")?;
            self.camera = Some(cam);
        }

        if let Some(cam) = &mut self.camera {
            let frame = cam.frame().context("Failed to capture frame")?;
            let buffer = frame.buffer();
            
            // Encode to JPEG using image crate
            let img = image::RgbImage::from_raw(frame.resolution().width(), frame.resolution().height(), buffer.to_vec())
                .context("Failed to parse raw buffer as RGB image")?;
                
            let mut cursor = std::io::Cursor::new(Vec::new());
            img.write_to(&mut cursor, image::ImageOutputFormat::Jpeg(80))
                .context("Failed to encode as JPEG")?;
            
            return Ok(cursor.into_inner());
        }
        
        anyhow::bail!("Camera is not initialized")
    }
}
