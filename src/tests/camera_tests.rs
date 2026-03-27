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
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
        0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 156, 99, 248, 207, 192, 240,
        31, 0, 5, 0, 1, 255, 137, 153, 61, 29, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ];
    let encoded_jpeg = state
        .convert_to_jpeg(&png, 1, 1)
        .expect("encoded image input should convert to jpeg");
    assert_eq!(&encoded_jpeg[0..2], &[0xFF, 0xD8]);
}
