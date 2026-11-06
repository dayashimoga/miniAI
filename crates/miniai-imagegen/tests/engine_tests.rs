use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use miniai_imagegen::{
    GenerationMode, ImageGenConfig, ImageGenEngine, ImageGenError, StepProgress,
};

#[test]
fn test_deterministic_synthesis_reproducibility() {
    let engine = ImageGenEngine::default();
    let config = ImageGenConfig {
        prompt: "A neon cyberpunk street at twilight".to_string(),
        negative_prompt: None,
        width: 128,
        height: 128,
        steps: 4,
        guidance_scale: 7.5,
        seed: 42,
        strength: 0.75,
        mode: GenerationMode::TextToImage,
    };

    let run1 = engine
        .generate_text_to_image::<fn(&StepProgress) -> bool>(&config, None, None)
        .expect("Run 1 must succeed");

    let run2 = engine
        .generate_text_to_image::<fn(&StepProgress) -> bool>(&config, None, None)
        .expect("Run 2 must succeed");

    assert_eq!(
        run1.provenance.sha256, run2.provenance.sha256,
        "Identical seed and prompt must produce identical SHA-256"
    );
    assert_eq!(run1.raw_rgb, run2.raw_rgb);
}

#[test]
fn test_seed_diversity() {
    let engine = ImageGenEngine::default();
    let mut config1 = ImageGenConfig {
        prompt: "A lush meadow with wild poppies".to_string(),
        negative_prompt: None,
        width: 128,
        height: 128,
        steps: 3,
        guidance_scale: 7.5,
        seed: 1001,
        strength: 0.75,
        mode: GenerationMode::TextToImage,
    };

    let run1 = engine
        .generate_text_to_image::<fn(&StepProgress) -> bool>(&config1, None, None)
        .unwrap();

    config1.seed = 2002;
    let run2 = engine
        .generate_text_to_image::<fn(&StepProgress) -> bool>(&config1, None, None)
        .unwrap();

    assert_ne!(
        run1.provenance.sha256, run2.provenance.sha256,
        "Different seeds must produce different image digests"
    );
}

#[test]
fn test_step_progress_telemetry() {
    let engine = ImageGenEngine::default();
    let config = ImageGenConfig {
        prompt: "Sunset over an ancient desert citadel".to_string(),
        negative_prompt: None,
        width: 128,
        height: 128,
        steps: 5,
        guidance_scale: 7.5,
        seed: 99,
        strength: 0.75,
        mode: GenerationMode::TextToImage,
    };

    let mut reported_steps = Vec::new();
    let res = engine.generate_text_to_image(
        &config,
        None,
        Some(|progress: &StepProgress| {
            reported_steps.push(progress.step);
            assert!(progress.progress_percent > 0.0);
            true
        }),
    );

    assert!(res.is_ok());
    assert_eq!(reported_steps, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_cooperative_cancellation_via_token() {
    let engine = ImageGenEngine::default();
    let config = ImageGenConfig {
        prompt: "Intricate clockwork mechanism".to_string(),
        negative_prompt: None,
        width: 128,
        height: 128,
        steps: 10,
        guidance_scale: 7.5,
        seed: 55,
        strength: 0.75,
        mode: GenerationMode::TextToImage,
    };

    let cancel_token = Arc::new(AtomicBool::new(false));
    let token_clone = Arc::clone(&cancel_token);

    let res = engine.generate_text_to_image(
        &config,
        Some(&cancel_token),
        Some(move |progress: &StepProgress| {
            if progress.step >= 3 {
                token_clone.store(true, Ordering::Relaxed);
            }
            true
        }),
    );

    assert_eq!(res.unwrap_err(), ImageGenError::GenerationCancelled);
}

#[test]
fn test_cooperative_cancellation_via_callback() {
    let engine = ImageGenEngine::default();
    let config = ImageGenConfig {
        prompt: "Starlit mountain summit".to_string(),
        negative_prompt: None,
        width: 128,
        height: 128,
        steps: 8,
        guidance_scale: 7.5,
        seed: 12,
        strength: 0.75,
        mode: GenerationMode::TextToImage,
    };

    let res = engine.generate_text_to_image(
        &config,
        None,
        Some(|progress: &StepProgress| {
            // Cancel at step 2 by returning false
            progress.step < 2
        }),
    );

    assert_eq!(res.unwrap_err(), ImageGenError::GenerationCancelled);
}

#[test]
fn test_inpainting_preserves_unmasked_pixels() {
    let engine = ImageGenEngine::default();
    let width = 128;
    let height = 128;
    let total_pixels = width * height;

    // Create solid green base image
    let mut base_rgb = Vec::with_capacity(total_pixels * 3);
    for _ in 0..total_pixels {
        base_rgb.extend_from_slice(&[20, 200, 20]);
    }

    // Left half (mask = 0, keep base), Right half (mask = 255, inpaint)
    let mut mask_bytes = Vec::with_capacity(total_pixels);
    for _ in 0..height {
        for x in 0..width {
            let mask = if x < width / 2 { 0 } else { 255 };
            mask_bytes.push(mask);
        }
    }

    let config = ImageGenConfig {
        prompt: "A blazing crimson crystal in a clearing".to_string(),
        negative_prompt: None,
        width: width as u32,
        height: height as u32,
        steps: 4,
        guidance_scale: 8.0,
        seed: 777,
        strength: 0.9,
        mode: GenerationMode::Inpainting,
    };

    let result = engine
        .inpaint_image::<fn(&StepProgress) -> bool>(&config, &base_rgb, &mask_bytes, None, None)
        .expect("Inpainting must succeed");

    // Check that left half (x < 64) is green (base preserved)
    for y in 0..height {
        for x in 0..(width / 2) {
            let idx = (y * width + x) * 3;
            assert_eq!(result.raw_rgb[idx], 20);
            assert_eq!(result.raw_rgb[idx + 1], 200);
            assert_eq!(result.raw_rgb[idx + 2], 20);
        }
    }

    // Check that right half (x >= 64) was synthesized (not solid green)
    let sample_right_idx = ((height / 2) * width + (width * 3 / 4)) * 3;
    let not_same_green = result.raw_rgb[sample_right_idx] != 20
        || result.raw_rgb[sample_right_idx + 1] != 200
        || result.raw_rgb[sample_right_idx + 2] != 20;
    assert!(not_same_green, "Masked region must be altered by synthesis");
}
