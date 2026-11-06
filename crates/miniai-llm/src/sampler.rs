use std::collections::HashMap;

/// Safe, deterministic pseudo-random number generator using XorShift64*.
#[derive(Debug, Clone)]
pub struct Prng {
    state: u64,
}

impl Prng {
    pub fn new(seed: u64) -> Self {
        // Ensure non-zero state
        let state = if seed == 0 { 0x853c49e6748fea9b } else { seed };
        Self { state }
    }

    /// Generates next 64-bit unsigned integer.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Generates a float in the half-open interval `[0.0, 1.0)`.
    pub fn next_f32(&mut self) -> f32 {
        let v = (self.next_u64() >> 40) as u32;
        v as f32 / 16777216.0
    }
}

/// Token sampling engine supporting greedy, temperature, top-p, and repetition penalty.
#[derive(Debug, Clone)]
pub struct Sampler {
    temperature: f32,
    top_p: f32,
    repetition_penalty: f32,
    prng: Prng,
}

impl Sampler {
    pub fn new(temperature: f32, top_p: f32, repetition_penalty: f32, seed: Option<u64>) -> Self {
        let seed_val = seed.unwrap_or(0x42_99_13_37);
        Self {
            temperature,
            top_p,
            repetition_penalty,
            prng: Prng::new(seed_val),
        }
    }

    /// Samples the next token ID from raw forward-pass logits.
    pub fn sample(&mut self, logits: &[f32], context_tokens: &[u32]) -> u32 {
        if logits.is_empty() {
            return 0;
        }

        // Greedy decoding if temperature is 0.0 or near zero
        if self.temperature <= 1e-6 {
            return self.sample_greedy(logits, context_tokens);
        }

        // Clone logits for in-place transformation
        let mut scaled_logits = logits.to_vec();

        // 1. Repetition penalty (scaled by token occurrence frequency)
        if (self.repetition_penalty - 1.0).abs() > 1e-4 {
            let mut counts: HashMap<u32, u32> = HashMap::new();
            for &t in context_tokens {
                *counts.entry(t).or_insert(0) += 1;
            }
            for (idx, logit) in scaled_logits.iter_mut().enumerate() {
                if let Some(&count) = counts.get(&(idx as u32)) {
                    let factor = self.repetition_penalty.powi(count.min(10) as i32);
                    if *logit > 0.0 {
                        *logit /= factor;
                    } else {
                        *logit *= factor;
                    }
                }
            }
        }

        // 2. Temperature scaling
        let inv_temp = 1.0 / self.temperature;
        for logit in &mut scaled_logits {
            *logit *= inv_temp;
        }

        // 3. Numerically stable softmax
        let max_logit = scaled_logits
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        let mut exps: Vec<f32> = scaled_logits
            .iter()
            .map(|&l| (l - max_logit).exp())
            .collect();
        let sum_exp: f32 = exps.iter().sum();
        if sum_exp > 0.0 {
            let inv_sum = 1.0 / sum_exp;
            for p in &mut exps {
                *p *= inv_sum;
            }
        } else {
            return self.sample_greedy(logits, context_tokens);
        }

        // 4. Top-p (nucleus) filtering
        let mut token_probs: Vec<(u32, f32)> = exps
            .iter()
            .copied()
            .enumerate()
            .map(|(id, prob)| (id as u32, prob))
            .collect();

        // Sort descending by probability
        token_probs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Truncate to top_p cumulative probability
        let mut cumulative = 0.0;
        let mut cutoff_index = token_probs.len();
        for (i, &(_, p)) in token_probs.iter().enumerate() {
            cumulative += p;
            if cumulative >= self.top_p {
                cutoff_index = (i + 1).min(token_probs.len());
                break;
            }
        }
        token_probs.truncate(cutoff_index);

        // Renormalize probabilities after truncation
        let filtered_sum: f32 = token_probs.iter().map(|(_, p)| *p).sum();
        if filtered_sum > 0.0 {
            let inv_fsum = 1.0 / filtered_sum;
            for (_, p) in &mut token_probs {
                *p *= inv_fsum;
            }
        }

        // 5. Discrete CDF sampling
        let r = self.prng.next_f32();
        let mut acc = 0.0;
        for (token_id, p) in &token_probs {
            acc += p;
            if r <= acc {
                return *token_id;
            }
        }

        // Fallback to highest probability token
        token_probs.first().map(|(id, _)| *id).unwrap_or(0)
    }

    fn sample_greedy(&self, logits: &[f32], context_tokens: &[u32]) -> u32 {
        let mut best_id = 0;
        let mut best_logit = f32::NEG_INFINITY;
        let mut counts: HashMap<u32, u32> = HashMap::new();
        for &t in context_tokens {
            *counts.entry(t).or_insert(0) += 1;
        }

        for (id, &logit) in logits.iter().enumerate() {
            let mut adjusted = logit;
            if (self.repetition_penalty - 1.0).abs() > 1e-4 {
                if let Some(&count) = counts.get(&(id as u32)) {
                    let factor = self.repetition_penalty.powi(count.min(10) as i32);
                    if adjusted > 0.0 {
                        adjusted /= factor;
                    } else {
                        adjusted *= factor;
                    }
                }
            }
            if adjusted > best_logit {
                best_logit = adjusted;
                best_id = id as u32;
            }
        }
        best_id
    }
}
