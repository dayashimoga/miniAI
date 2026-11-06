use std::io::Write;
use tempfile::NamedTempFile;

use miniai_core::{AcceleratorBackend, Modality, ModalityAdapter};
use miniai_imagegen::{
    GenerationMode, ImageGenAdapter, ImageGenConfig, ImageGenError, StepProgress,
};
use sha2::{Digest, Sha256};

#[test]
fn test_imagegen_adapter_modality_contract() {
    let adapter = ImageGenAdapter::new("imagegen_test_adapter");
    assert_eq!(adapter.adapter_id(), "imagegen_test_adapter");
    assert_eq!(adapter.modality(), Modality::ImageGeneration);
    assert_eq!(adapter.supported_backends(), vec![AcceleratorBackend::Cpu]);

    let cap = adapter.capability();
    assert_eq!(cap.modality, Modality::ImageGeneration);
    assert!(cap.supports_offline);
    assert!(cap.isolated_process);

    let health = adapter.health_check().expect("Health check must succeed");
    assert!(health.ready);
    assert_eq!(health.active_backend, AcceleratorBackend::Cpu);
}

#[test]
fn test_imagegen_adapter_weights_loading_and_sha256() {
    let adapter = ImageGenAdapter::new("imagegen_weights_test");

    let weights_content = b"MINIAI_DIFFUSION_CPU_WEIGHTS_MOCK_PAYLOAD_V1";
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(weights_content).unwrap();

    let mut hasher = Sha256::new();
    hasher.update(weights_content);
    let expected_sha256 = format!("{:x}", hasher.finalize());

    // 1. Valid load
    adapter
        .load_model_weights(file.path(), Some(&expected_sha256))
        .expect("Valid model load must succeed");
    assert!(adapter.is_loaded());

    // 2. Mismatched digest
    let err = adapter
        .load_model_weights(
            file.path(),
            Some("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
        )
        .unwrap_err();
    assert!(matches!(err, ImageGenError::PngError { .. }));

    // 3. Unload model
    adapter.unload_model();
    assert!(!adapter.is_loaded());
    let health = adapter.health_check().unwrap();
    assert!(!health.ready);
}

#[test]
fn test_imagegen_adapter_end_to_end_operations() {
    let adapter = ImageGenAdapter::new("imagegen_e2e_adapter");

    let config = ImageGenConfig {
        prompt: "A mystical forest bathed in moonlight".to_string(),
        negative_prompt: Some("blurry".to_string()),
        width: 128,
        height: 128,
        steps: 3,
        guidance_scale: 7.0,
        seed: 314,
        strength: 0.8,
        mode: GenerationMode::TextToImage,
    };

    // 1. Text-to-Image
    let t2i_result = adapter
        .generate::<fn(&StepProgress) -> bool>(&config, None, None)
        .expect("Generate must succeed");
    assert!(t2i_result.png_bytes.len() > 100);
    assert_eq!(t2i_result.provenance.width, 128);
    assert_eq!(t2i_result.provenance.height, 128);

    // 2. Image-to-Image refinement
    let i2i_config = ImageGenConfig {
        mode: GenerationMode::ImageToImage,
        ..config.clone()
    };
    let i2i_result = adapter
        .refine::<fn(&StepProgress) -> bool>(&i2i_config, &t2i_result.raw_rgb, 128, 128, None, None)
        .expect("Refine must succeed");
    assert!(i2i_result.png_bytes.len() > 100);

    // 3. Inpainting
    let inpaint_config = ImageGenConfig {
        mode: GenerationMode::Inpainting,
        ..config
    };
    let mask = vec![128u8; 128 * 128]; // semi-masked
    let inpaint_result = adapter
        .inpaint::<fn(&StepProgress) -> bool>(
            &inpaint_config,
            &t2i_result.raw_rgb,
            &mask,
            None,
            None,
        )
        .expect("Inpaint must succeed");
    assert!(inpaint_result.png_bytes.len() > 100);
}
