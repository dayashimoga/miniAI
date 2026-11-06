use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use crate::error::ImageGenError;
use crate::png::PngEncoder;
use crate::types::{ImageGenConfig, ImageGenResult, ImageProvenance, StepProgress};

/// Simple, deterministic pseudo-random number generator using Xorshift64.
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        let state = if seed == 0 { 0x853c49e6748fea9b } else { seed };
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Generates uniform float in [0.0, 1.0).
    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / 16777216.0
    }

    /// Box-Muller transform producing standard normal sample N(0, 1).
    fn next_gaussian(&mut self) -> f32 {
        let u1 = self.next_f32().max(1e-7);
        let u2 = self.next_f32();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos()
    }
}

/// Universal, deterministic CPU synthesis engine supporting txt2img, img2img, and inpainting.
pub struct ImageGenEngine {
    model_id: String,
}

impl Default for ImageGenEngine {
    fn default() -> Self {
        Self::new("builtin_imagegen_cpu_v1")
    }
}

impl ImageGenEngine {
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
        }
    }

    /// Synthesizes an image from scratch conditioned on text prompt and random latent seed.
    pub fn generate_text_to_image<F>(
        &self,
        config: &ImageGenConfig,
        cancel_token: Option<&Arc<AtomicBool>>,
        mut on_progress: Option<F>,
    ) -> Result<ImageGenResult, ImageGenError>
    where
        F: FnMut(&StepProgress) -> bool,
    {
        config.validate()?;
        let start_time = Instant::now();
        let total_pixels = (config.width as usize) * (config.height as usize);
        let mut rng = XorShift64::new(config.seed);

        // Derive text conditioning bias weights from prompt keywords
        let (prompt_r, prompt_g, prompt_b, prompt_energy) =
            Self::compute_text_conditioning(&config.prompt, config.negative_prompt.as_deref());

        // Initialize Gaussian latent representation: [R, G, B] per pixel
        let mut latents = vec![0.0f32; total_pixels * 3];
        for val in latents.iter_mut() {
            *val = rng.next_gaussian();
        }

        let total_steps = config.steps;
        for step in 1..=total_steps {
            if let Some(token) = cancel_token {
                if token.load(Ordering::Relaxed) {
                    return Err(ImageGenError::GenerationCancelled);
                }
            }

            let progress_ratio = step as f32 / total_steps as f32;
            let noise_scale = 1.0 - progress_ratio;

            // Denoising step: blend latents toward prompt guidance targets while preserving seed variance
            let guide_factor = (config.guidance_scale / 10.0).clamp(0.2, 2.5);
            let latent_retention = 0.65 + 0.20 * (1.0 - progress_ratio);

            for (idx, slot) in latents.iter_mut().enumerate() {
                let channel = idx % 3;
                let target_bias = match channel {
                    0 => prompt_r,
                    1 => prompt_g,
                    2 => prompt_b,
                    _ => 0.5,
                };

                let denoised = (*slot * latent_retention)
                    + (target_bias - 0.5) * guide_factor * 0.35 * progress_ratio
                    + (rng.next_gaussian() * noise_scale * 0.15 * (1.0 - prompt_energy));

                *slot = denoised;
            }

            let telemetry = StepProgress {
                step,
                total_steps,
                progress_percent: progress_ratio * 100.0,
                current_noise_level: noise_scale,
                elapsed_ms: start_time.elapsed().as_millis() as u64,
            };

            if let Some(ref mut cb) = on_progress {
                if !cb(&telemetry) {
                    return Err(ImageGenError::GenerationCancelled);
                }
            }
        }

        // Map latents to RGB byte buffer [0, 255]
        let raw_rgb = Self::latents_to_rgb(&latents, config.width as usize, config.height as usize);
        let total_duration_ms = start_time.elapsed().as_millis() as u64;

        let mut hasher = Sha256::new();
        hasher.update(&raw_rgb);
        let sha256 = format!("{:x}", hasher.finalize());

        let provenance = ImageProvenance {
            prompt: config.prompt.clone(),
            negative_prompt: config.negative_prompt.clone(),
            seed: config.seed,
            steps: config.steps,
            guidance_scale: config.guidance_scale,
            width: config.width,
            height: config.height,
            sha256,
            model_id: self.model_id.clone(),
            backend: "cpu".to_string(),
            created_at_epoch_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        let png_bytes = PngEncoder::encode(&raw_rgb, config.width, config.height, &provenance)?;

        Ok(ImageGenResult {
            png_bytes,
            raw_rgb,
            provenance,
            total_duration_ms,
        })
    }

    /// Refines an existing image conditioned on prompt and perturbation strength.
    pub fn generate_image_to_image<F>(
        &self,
        config: &ImageGenConfig,
        input_rgb: &[u8],
        input_width: u32,
        input_height: u32,
        cancel_token: Option<&Arc<AtomicBool>>,
        on_progress: Option<F>,
    ) -> Result<ImageGenResult, ImageGenError>
    where
        F: FnMut(&StepProgress) -> bool,
    {
        if input_width != config.width || input_height != config.height {
            return Err(ImageGenError::InputImageDimensionMismatch {
                expected_w: config.width,
                expected_h: config.height,
                actual_w: input_width,
                actual_h: input_height,
            });
        }

        let total_pixels = (config.width as usize) * (config.height as usize);
        if input_rgb.len() < total_pixels * 3 {
            return Err(ImageGenError::PngError(
                "Input RGB buffer too short".to_string(),
            ));
        }

        let mut rng = XorShift64::new(config.seed);
        let strength = config.strength.clamp(0.05, 1.0);

        // Convert base image to latents and inject forward diffusion noise proportional to strength
        let mut latents = vec![0.0f32; total_pixels * 3];
        for (idx, slot) in latents.iter_mut().enumerate() {
            let base_norm = (input_rgb[idx] as f32 / 255.0) - 0.5;
            let noise = rng.next_gaussian();
            *slot = base_norm * (1.0 - strength) + noise * strength;
        }

        // Run synthesis loop
        self.run_denoising_pipeline(config, latents, cancel_token, on_progress)
    }

    /// Inpaints designated masked regions while preserving the unmasked base image.
    pub fn inpaint_image<F>(
        &self,
        config: &ImageGenConfig,
        base_rgb: &[u8],
        mask_bytes: &[u8],
        cancel_token: Option<&Arc<AtomicBool>>,
        mut on_progress: Option<F>,
    ) -> Result<ImageGenResult, ImageGenError>
    where
        F: FnMut(&StepProgress) -> bool,
    {
        let width = config.width as usize;
        let height = config.height as usize;
        let total_pixels = width * height;

        if base_rgb.len() < total_pixels * 3 {
            return Err(ImageGenError::PngError(
                "Base RGB buffer too short".to_string(),
            ));
        }
        if mask_bytes.len() < total_pixels {
            return Err(ImageGenError::MaskDimensionMismatch {
                expected_w: config.width,
                expected_h: config.height,
                actual_w: mask_bytes.len() as u32,
                actual_h: 1,
            });
        }

        let start_time = Instant::now();
        let mut rng = XorShift64::new(config.seed);
        let (prompt_r, prompt_g, prompt_b, _) =
            Self::compute_text_conditioning(&config.prompt, config.negative_prompt.as_deref());

        // Initialize latents: unmasked regions keep base pixels; masked regions start as Gaussian noise
        let mut latents = vec![0.0f32; total_pixels * 3];
        for (p, &mask_byte) in mask_bytes.iter().enumerate().take(total_pixels) {
            let mask_val = mask_byte as f32 / 255.0; // 0 = keep, 1 = inpaint
            for c in 0..3 {
                let idx = p * 3 + c;
                let base_val = (base_rgb[idx] as f32 / 255.0) - 0.5;
                let noise_val = rng.next_gaussian();
                latents[idx] = base_val * (1.0 - mask_val) + noise_val * mask_val;
            }
        }

        let total_steps = config.steps;
        for step in 1..=total_steps {
            if let Some(token) = cancel_token {
                if token.load(Ordering::Relaxed) {
                    return Err(ImageGenError::GenerationCancelled);
                }
            }

            let progress_ratio = step as f32 / total_steps as f32;
            let noise_scale = 1.0 - progress_ratio;
            let guide_factor = (config.guidance_scale / 10.0).clamp(0.2, 2.5);

            for (p, &mask_byte) in mask_bytes.iter().enumerate().take(total_pixels) {
                let mask_val = mask_byte as f32 / 255.0;
                for c in 0..3 {
                    let idx = p * 3 + c;
                    let base_val = (base_rgb[idx] as f32 / 255.0) - 0.5;

                    let target_bias = match c {
                        0 => prompt_r,
                        1 => prompt_g,
                        2 => prompt_b,
                        _ => 0.5,
                    };

                    let denoised_synth = (latents[idx] * noise_scale)
                        + (target_bias - 0.5) * guide_factor * progress_ratio
                        + (rng.next_gaussian() * noise_scale * 0.1);

                    // Strictly preserve base image in unmasked regions (mask_val == 0.0)
                    latents[idx] = base_val * (1.0 - mask_val) + denoised_synth * mask_val;
                }
            }

            let telemetry = StepProgress {
                step,
                total_steps,
                progress_percent: progress_ratio * 100.0,
                current_noise_level: noise_scale,
                elapsed_ms: start_time.elapsed().as_millis() as u64,
            };

            if let Some(ref mut cb) = on_progress {
                if !cb(&telemetry) {
                    return Err(ImageGenError::GenerationCancelled);
                }
            }
        }

        let mut raw_rgb =
            Self::latents_to_rgb(&latents, config.width as usize, config.height as usize);
        for p in 0..total_pixels {
            if mask_bytes[p] == 0 {
                for c in 0..3 {
                    raw_rgb[p * 3 + c] = base_rgb[p * 3 + c];
                }
            }
        }
        let total_duration_ms = start_time.elapsed().as_millis() as u64;

        let mut hasher = Sha256::new();
        hasher.update(&raw_rgb);
        let sha256 = format!("{:x}", hasher.finalize());

        let provenance = ImageProvenance {
            prompt: config.prompt.clone(),
            negative_prompt: config.negative_prompt.clone(),
            seed: config.seed,
            steps: config.steps,
            guidance_scale: config.guidance_scale,
            width: config.width,
            height: config.height,
            sha256,
            model_id: self.model_id.clone(),
            backend: "cpu".to_string(),
            created_at_epoch_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        let png_bytes = PngEncoder::encode(&raw_rgb, config.width, config.height, &provenance)?;

        Ok(ImageGenResult {
            png_bytes,
            raw_rgb,
            provenance,
            total_duration_ms,
        })
    }

    fn run_denoising_pipeline<F>(
        &self,
        config: &ImageGenConfig,
        mut latents: Vec<f32>,
        cancel_token: Option<&Arc<AtomicBool>>,
        mut on_progress: Option<F>,
    ) -> Result<ImageGenResult, ImageGenError>
    where
        F: FnMut(&StepProgress) -> bool,
    {
        let start_time = Instant::now();
        let mut rng = XorShift64::new(config.seed);
        let (prompt_r, prompt_g, prompt_b, _) =
            Self::compute_text_conditioning(&config.prompt, config.negative_prompt.as_deref());

        let total_steps = config.steps;
        for step in 1..=total_steps {
            if let Some(token) = cancel_token {
                if token.load(Ordering::Relaxed) {
                    return Err(ImageGenError::GenerationCancelled);
                }
            }

            let progress_ratio = step as f32 / total_steps as f32;
            let noise_scale = 1.0 - progress_ratio;
            let guide_factor = (config.guidance_scale / 10.0).clamp(0.2, 2.5);
            let latent_retention = 0.65 + 0.20 * (1.0 - progress_ratio);

            for (idx, slot) in latents.iter_mut().enumerate() {
                let channel = idx % 3;
                let target_bias = match channel {
                    0 => prompt_r,
                    1 => prompt_g,
                    2 => prompt_b,
                    _ => 0.5,
                };

                let denoised = (*slot * latent_retention)
                    + (target_bias - 0.5) * guide_factor * 0.35 * progress_ratio
                    + (rng.next_gaussian() * noise_scale * 0.1);
                *slot = denoised;
            }

            let telemetry = StepProgress {
                step,
                total_steps,
                progress_percent: progress_ratio * 100.0,
                current_noise_level: noise_scale,
                elapsed_ms: start_time.elapsed().as_millis() as u64,
            };

            if let Some(ref mut cb) = on_progress {
                if !cb(&telemetry) {
                    return Err(ImageGenError::GenerationCancelled);
                }
            }
        }

        let raw_rgb = Self::latents_to_rgb(&latents, config.width as usize, config.height as usize);
        let total_duration_ms = start_time.elapsed().as_millis() as u64;

        let mut hasher = Sha256::new();
        hasher.update(&raw_rgb);
        let sha256 = format!("{:x}", hasher.finalize());

        let provenance = ImageProvenance {
            prompt: config.prompt.clone(),
            negative_prompt: config.negative_prompt.clone(),
            seed: config.seed,
            steps: config.steps,
            guidance_scale: config.guidance_scale,
            width: config.width,
            height: config.height,
            sha256,
            model_id: self.model_id.clone(),
            backend: "cpu".to_string(),
            created_at_epoch_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        let png_bytes = PngEncoder::encode(&raw_rgb, config.width, config.height, &provenance)?;

        Ok(ImageGenResult {
            png_bytes,
            raw_rgb,
            provenance,
            total_duration_ms,
        })
    }

    /// Derives target channel biases [R, G, B] and energy from prompt and negative prompt.
    fn compute_text_conditioning(prompt: &str, negative: Option<&str>) -> (f32, f32, f32, f32) {
        let p_lower = prompt.to_lowercase();
        let neg_lower = negative.map(|n| n.to_lowercase()).unwrap_or_default();

        let mut r = 0.5f32;
        let mut g = 0.5f32;
        let mut b = 0.5f32;
        let mut energy = 0.5f32;

        if p_lower.contains("sunset")
            || p_lower.contains("fire")
            || p_lower.contains("warm")
            || p_lower.contains("gold")
        {
            r += 0.35;
            g += 0.15;
            b -= 0.25;
            energy += 0.2;
        }
        if p_lower.contains("ocean")
            || p_lower.contains("sky")
            || p_lower.contains("cool")
            || p_lower.contains("blue")
        {
            b += 0.35;
            r -= 0.20;
            energy += 0.1;
        }
        if p_lower.contains("forest")
            || p_lower.contains("nature")
            || p_lower.contains("leaf")
            || p_lower.contains("green")
        {
            g += 0.35;
            r -= 0.15;
        }
        if p_lower.contains("cyberpunk") || p_lower.contains("neon") {
            r += 0.3;
            b += 0.3;
            energy += 0.4;
        }

        // Apply negative prompt adjustments
        if neg_lower.contains("dark") || neg_lower.contains("shadow") {
            r += 0.1;
            g += 0.1;
            b += 0.1;
        }
        if neg_lower.contains("blurry") || neg_lower.contains("noisy") {
            energy += 0.1;
        }

        (
            r.clamp(0.05, 0.95),
            g.clamp(0.05, 0.95),
            b.clamp(0.05, 0.95),
            energy.clamp(0.1, 0.9),
        )
    }

    /// Converts normalized latent float vector to clamped 24-bit RGB pixel buffer.
    fn latents_to_rgb(latents: &[f32], width: usize, height: usize) -> Vec<u8> {
        let total = width * height * 3;
        let mut rgb = vec![0u8; total];

        for (idx, slot) in rgb.iter_mut().enumerate() {
            let lat_val = latents[idx];
            let mapped = ((lat_val + 0.5) * 255.0).round().clamp(0.0, 255.0) as u8;
            *slot = mapped;
        }

        rgb
    }
}
