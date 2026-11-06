use miniai_imagegen::{GenerationMode, ImageGenConfig, ImageGenError};

#[test]
fn test_default_config_validates() {
    let config = ImageGenConfig {
        prompt: "A picturesque mountain peak at golden hour".to_string(),
        ..Default::default()
    };
    assert!(config.validate().is_ok());
}

#[test]
fn test_reject_empty_prompt() {
    let config = ImageGenConfig {
        prompt: "   ".to_string(),
        ..Default::default()
    };
    let err = config.validate().unwrap_err();
    assert!(matches!(err, ImageGenError::InvalidDimension { .. }));
}

#[test]
fn test_dimension_validation_multiples_of_64() {
    // Valid multiples of 64
    let valid_config = ImageGenConfig {
        prompt: "Cyberpunk cityscape".to_string(),
        width: 256,
        height: 768,
        ..Default::default()
    };
    assert!(valid_config.validate().is_ok());

    // Invalid non-multiple of 64
    let invalid_multiple = ImageGenConfig {
        prompt: "Cyberpunk cityscape".to_string(),
        width: 500,
        ..Default::default()
    };
    let err = invalid_multiple.validate().unwrap_err();
    assert!(matches!(err, ImageGenError::InvalidDimension { .. }));

    // Below minimum
    let below_min = ImageGenConfig {
        prompt: "Cyberpunk cityscape".to_string(),
        width: 64,
        height: 512,
        ..Default::default()
    };
    let err = below_min.validate().unwrap_err();
    assert!(matches!(err, ImageGenError::InvalidDimension { .. }));

    // Above maximum
    let above_max = ImageGenConfig {
        prompt: "Cyberpunk cityscape".to_string(),
        width: 4096,
        ..Default::default()
    };
    let err = above_max.validate().unwrap_err();
    assert!(matches!(err, ImageGenError::InvalidDimension { .. }));
}

#[test]
fn test_steps_validation() {
    let zero_steps = ImageGenConfig {
        prompt: "Lush rainforest".to_string(),
        steps: 0,
        ..Default::default()
    };
    assert!(matches!(
        zero_steps.validate().unwrap_err(),
        ImageGenError::InvalidStepCount { .. }
    ));

    let excessive_steps = ImageGenConfig {
        prompt: "Lush rainforest".to_string(),
        steps: 101,
        ..Default::default()
    };
    assert!(matches!(
        excessive_steps.validate().unwrap_err(),
        ImageGenError::InvalidStepCount { .. }
    ));

    let valid_steps = ImageGenConfig {
        prompt: "Lush rainforest".to_string(),
        steps: 50,
        ..Default::default()
    };
    assert!(valid_steps.validate().is_ok());
}

#[test]
fn test_guidance_scale_validation() {
    let low_guidance = ImageGenConfig {
        prompt: "Watercolor harbor".to_string(),
        guidance_scale: 0.5,
        ..Default::default()
    };
    assert!(matches!(
        low_guidance.validate().unwrap_err(),
        ImageGenError::InvalidGuidanceScale { .. }
    ));

    let high_guidance = ImageGenConfig {
        prompt: "Watercolor harbor".to_string(),
        guidance_scale: 30.0,
        ..Default::default()
    };
    assert!(matches!(
        high_guidance.validate().unwrap_err(),
        ImageGenError::InvalidGuidanceScale { .. }
    ));

    let valid_guidance = ImageGenConfig {
        prompt: "Watercolor harbor".to_string(),
        guidance_scale: 12.0,
        ..Default::default()
    };
    assert!(valid_guidance.validate().is_ok());
}

#[test]
fn test_strength_validation_for_img2img() {
    let low_strength = ImageGenConfig {
        prompt: "Oil painting conversion".to_string(),
        mode: GenerationMode::ImageToImage,
        strength: 0.01,
        ..Default::default()
    };
    assert!(matches!(
        low_strength.validate().unwrap_err(),
        ImageGenError::InvalidStrength { .. }
    ));

    let high_strength = ImageGenConfig {
        prompt: "Oil painting conversion".to_string(),
        mode: GenerationMode::ImageToImage,
        strength: 1.5,
        ..Default::default()
    };
    assert!(matches!(
        high_strength.validate().unwrap_err(),
        ImageGenError::InvalidStrength { .. }
    ));

    let valid_strength = ImageGenConfig {
        prompt: "Oil painting conversion".to_string(),
        mode: GenerationMode::ImageToImage,
        strength: 0.8,
        ..Default::default()
    };
    assert!(valid_strength.validate().is_ok());
}
