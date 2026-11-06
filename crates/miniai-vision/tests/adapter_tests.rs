use std::io::Write;
use tempfile::NamedTempFile;

use miniai_core::{AcceleratorBackend, Modality, ModalityAdapter};
use miniai_vision::{VisionAdapter, VisionError, VisualQaQuery};
use sha2::{Digest, Sha256};

fn create_test_png_bytes(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    bytes.extend_from_slice(&13u32.to_be_bytes());
    bytes.extend_from_slice(b"IHDR");
    bytes.extend_from_slice(&width.to_be_bytes());
    bytes.extend_from_slice(&height.to_be_bytes());
    bytes.push(8);
    bytes.push(2); // RGB
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

#[test]
fn test_vision_adapter_modality_contract() {
    let adapter = VisionAdapter::new("vision_test_adapter");
    assert_eq!(adapter.adapter_id(), "vision_test_adapter");
    assert_eq!(adapter.modality(), Modality::Vision);
    assert_eq!(adapter.supported_backends(), vec![AcceleratorBackend::Cpu]);

    let cap = adapter.capability();
    assert_eq!(cap.modality, Modality::Vision);
    assert!(cap.supports_offline);
    assert!(cap.isolated_process);

    let health = adapter.health_check().expect("Health check must succeed");
    assert!(health.ready);
    assert_eq!(health.active_backend, AcceleratorBackend::Cpu);
}

#[test]
fn test_vision_adapter_weights_loading_and_sha256_verification() {
    let adapter = VisionAdapter::new("vision_weights_test");

    let weights_content = b"MINIAI_VISION_CPU_WEIGHTS_MOCK_PAYLOAD_V1";
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(weights_content).unwrap();

    let mut hasher = Sha256::new();
    hasher.update(weights_content);
    let expected_sha256 = format!("{:x}", hasher.finalize());

    // 1. Valid load with matching digest
    adapter
        .load_model_weights(file.path(), Some(&expected_sha256))
        .expect("Valid model load must succeed");
    assert!(adapter.is_loaded());

    // 2. Load with mismatched digest must fail
    let err = adapter
        .load_model_weights(
            file.path(),
            Some("0000000000000000000000000000000000000000000000000000000000000000"),
        )
        .unwrap_err();
    assert!(matches!(err, VisionError::CorruptPayload { .. }));

    // 3. Unload model
    adapter.unload_model();
    assert!(!adapter.is_loaded());
    let health = adapter.health_check().unwrap();
    assert!(!health.ready);
}

#[test]
fn test_vision_adapter_end_to_end_operations() {
    let adapter = VisionAdapter::new("vision_e2e_adapter").with_resolution(48);

    let png_bytes = create_test_png_bytes(64, 64);
    let mut raw_rgb = Vec::with_capacity(64 * 64 * 3);
    for y in 0..64 {
        let val = if y < 32 { 220 } else { 30 };
        for _ in 0..64 {
            raw_rgb.extend_from_slice(&[val, val, val]);
        }
    }

    // 1. Sanitize image
    let meta = adapter
        .sanitize_image(&png_bytes)
        .expect("Sanitize must succeed");
    assert_eq!(meta.dimensions.width, 64);
    assert_eq!(meta.dimensions.height, 64);

    // 2. Describe image
    let desc = adapter
        .describe_image(&png_bytes, &raw_rgb)
        .expect("Describe must succeed");
    assert!(desc.caption.contains("64x64"));
    assert!(!desc.detected_tags.is_empty());

    // 3. Extract text OCR
    let ocr = adapter
        .extract_text_ocr(&png_bytes, &raw_rgb)
        .expect("OCR must succeed");
    assert!(!ocr.elements.is_empty());

    // 4. Answer VQA query
    let query = VisualQaQuery {
        prompt: "What are the dimensions of this image?".to_string(),
    };
    let answer = adapter
        .answer_vqa(&png_bytes, &raw_rgb, &query)
        .expect("VQA must succeed");
    assert!(answer.answer.contains("64x64"));
    assert!(!answer.grounding_evidence.is_empty());
}
