use miniai_vision::{
    ImageDimensions, ImageFormat, ImageSanitizer, VisionError, MAX_IMAGE_DIMENSION,
    MAX_IMAGE_FILE_SIZE_BYTES,
};

fn create_test_png(width: u32, height: u32, rgba: bool) -> Vec<u8> {
    let mut bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    bytes.extend_from_slice(&13u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.push(8);
    bytes.push(if rgba { 6 } else { 2 });
    bytes.push(0);
    bytes.push(0);
    bytes.push(0);
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes.extend_from_slice(&4u32.to_be_bytes());
    bytes.extend_from_slice(b"IDAT");
    bytes.extend_from_slice(&[0x78, 0x9C, 0x01, 0x00]);
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes.extend_from_slice(&0u32.to_be_bytes());
    bytes.extend_from_slice(b"IEND");
    bytes.extend_from_slice(&[0xAE, 0x42, 0x60, 0x82]);
    bytes
}

fn create_test_jpeg(width: u16, height: u16) -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xD8, 0xFF, 0xC0];
    let seg_len = 11u16;
    bytes.extend_from_slice(&seg_len.to_be_bytes());
    bytes.push(8);
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.push(3);
    bytes.extend_from_slice(&[1, 0x11, 0, 2, 0x11, 0, 3, 0x11, 0]);
    bytes.extend_from_slice(&[0xFF, 0xD9]);
    bytes
}

fn create_test_webp_lossless(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    let file_size = 32u32;
    bytes.extend_from_slice(&file_size.to_le_bytes());
    bytes.extend_from_slice(b"WEBP");
    bytes.extend_from_slice(b"VP8L");
    let chunk_size = 12u32;
    bytes.extend_from_slice(&chunk_size.to_le_bytes());
    bytes.push(0x2F);
    let w_sub = width - 1;
    let h_sub = height - 1;
    let b1 = (w_sub & 0xFF) as u8;
    let b2 = (((w_sub >> 8) & 0x3F) | ((h_sub & 0x03) << 6)) as u8;
    let b3 = ((h_sub >> 2) & 0xFF) as u8;
    let b4 = (((h_sub >> 10) & 0x0F) | 0x10) as u8;
    bytes.extend_from_slice(&[b1, b2, b3, b4]);
    bytes.extend_from_slice(&[0, 0, 0, 0]);
    bytes
}

#[test]
fn test_sanitize_valid_png() {
    let png = create_test_png(640, 480, true);
    let meta = ImageSanitizer::sanitize(&png).expect("PNG sanitization must succeed");

    assert_eq!(meta.format, ImageFormat::Png);
    assert_eq!(meta.dimensions, ImageDimensions::new(640, 480));
    assert!(meta.has_alpha);
    assert_eq!(meta.size_bytes, png.len());
    assert_eq!(meta.sha256.len(), 64);
}

#[test]
fn test_sanitize_valid_jpeg() {
    let jpeg = create_test_jpeg(1024, 768);
    let meta = ImageSanitizer::sanitize(&jpeg).expect("JPEG sanitization must succeed");

    assert_eq!(meta.format, ImageFormat::Jpeg);
    assert_eq!(meta.dimensions, ImageDimensions::new(1024, 768));
    assert!(!meta.has_alpha);
    assert_eq!(meta.dimensions.aspect_ratio_label(), "Landscape");
}

#[test]
fn test_sanitize_valid_webp_lossless() {
    let webp = create_test_webp_lossless(300, 600);
    let meta = ImageSanitizer::sanitize(&webp).expect("WebP sanitization must succeed");

    assert_eq!(meta.format, ImageFormat::WebP);
    assert_eq!(meta.dimensions, ImageDimensions::new(300, 600));
    assert!(meta.has_alpha);
    assert_eq!(meta.dimensions.aspect_ratio_label(), "Portrait");
}

#[test]
fn test_reject_invalid_magic_bytes() {
    let junk = vec![
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
    ];
    let err = ImageSanitizer::sanitize(&junk).unwrap_err();
    assert!(matches!(err, VisionError::InvalidMagicBytes { .. }));
}

#[test]
fn test_reject_short_payload() {
    let short = vec![0x89, 0x50, 0x4E];
    let err = ImageSanitizer::sanitize(&short).unwrap_err();
    assert!(matches!(err, VisionError::CorruptPayload { .. }));
}

#[test]
fn test_reject_oversized_dimensions() {
    let huge_png = create_test_png(MAX_IMAGE_DIMENSION + 1, 100, false);
    let err = ImageSanitizer::sanitize(&huge_png).unwrap_err();
    assert!(matches!(err, VisionError::DimensionsExceeded { .. }));
}

#[test]
fn test_reject_file_too_large() {
    // Generate simulated image exceeding 25 MB
    let mut huge_payload = create_test_png(100, 100, false);
    huge_payload.resize(MAX_IMAGE_FILE_SIZE_BYTES + 1, 0xAA);
    let err = ImageSanitizer::sanitize(&huge_payload).unwrap_err();
    assert!(matches!(err, VisionError::FileTooLarge { .. }));
}

#[test]
fn test_reject_decompression_bomb() {
    // A tiny file (60 bytes) claiming 4000x4000 = 16 million pixels
    let bomb_png = create_test_png(4000, 4000, false);
    let err = ImageSanitizer::sanitize(&bomb_png).unwrap_err();
    assert!(matches!(err, VisionError::DecompressionBomb { .. }));
}
