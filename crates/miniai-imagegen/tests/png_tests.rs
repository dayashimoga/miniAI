use miniai_imagegen::{
    calculate_adler32, calculate_crc32, ImageProvenance, PngDecoder, PngEncoder,
};

#[test]
fn test_checksum_algorithms() {
    // Standard test vector for "123456789"
    let test_bytes = b"123456789";
    let crc = calculate_crc32(&[test_bytes]);
    assert_eq!(crc, 0xCBF43926, "CRC-32 must match standard test vector");

    let adler = calculate_adler32(test_bytes);
    assert_eq!(
        adler, 0x091E01DE,
        "Adler-32 must match RFC 1950 test vector"
    );
}

#[test]
fn test_png_roundtrip_encoding_and_provenance_extraction() {
    let width = 64;
    let height = 64;
    let total_pixels = width * height;
    let mut rgb = Vec::with_capacity(total_pixels * 3);

    for y in 0..height {
        for x in 0..width {
            let r = ((x * 4) % 256) as u8;
            let g = ((y * 4) % 256) as u8;
            let b = (((x + y) * 2) % 256) as u8;
            rgb.extend_from_slice(&[r, g, b]);
        }
    }

    let provenance = ImageProvenance {
        prompt: "A serene Japanese zen garden in autumn".to_string(),
        negative_prompt: Some("blurry, low resolution".to_string()),
        seed: 1337,
        steps: 25,
        guidance_scale: 8.5,
        width: width as u32,
        height: height as u32,
        sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        model_id: "zen_diffusion_v1".to_string(),
        backend: "cpu".to_string(),
        created_at_epoch_secs: 1700000000,
    };

    let png_bytes = PngEncoder::encode(&rgb, width as u32, height as u32, &provenance)
        .expect("PNG encoding must succeed");

    assert!(png_bytes.len() > 8);
    assert_eq!(
        &png_bytes[0..8],
        &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
    );

    // Extract and verify provenance
    let decoded_provenance =
        PngDecoder::extract_provenance(&png_bytes).expect("Provenance extraction must succeed");

    assert_eq!(decoded_provenance.prompt, provenance.prompt);
    assert_eq!(
        decoded_provenance.negative_prompt,
        provenance.negative_prompt
    );
    assert_eq!(decoded_provenance.seed, 1337);
    assert_eq!(decoded_provenance.steps, 25);
    assert!((decoded_provenance.guidance_scale - 8.5).abs() < 1e-2);
    assert_eq!(decoded_provenance.width, width as u32);
    assert_eq!(decoded_provenance.height, height as u32);
    assert_eq!(decoded_provenance.sha256, provenance.sha256);
    assert_eq!(decoded_provenance.model_id, "zen_diffusion_v1");
    assert_eq!(decoded_provenance.backend, "cpu");
    assert_eq!(decoded_provenance.created_at_epoch_secs, 1700000000);
}

#[test]
fn test_png_underflow_error() {
    let incomplete_rgb = vec![255, 255];
    let provenance = ImageProvenance {
        prompt: "test".to_string(),
        negative_prompt: None,
        seed: 1,
        steps: 1,
        guidance_scale: 1.0,
        width: 64,
        height: 64,
        sha256: "test".to_string(),
        model_id: "test".to_string(),
        backend: "cpu".to_string(),
        created_at_epoch_secs: 0,
    };

    let res = PngEncoder::encode(&incomplete_rgb, 64, 64, &provenance);
    assert!(res.is_err());
}
