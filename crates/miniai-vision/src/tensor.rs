use crate::error::VisionError;

/// Standard ImageNet channel means for tensor normalization [R, G, B].
pub const IMAGENET_MEAN: [f32; 3] = [0.485, 0.456, 0.406];

/// Standard ImageNet channel standard deviations for tensor normalization [R, G, B].
pub const IMAGENET_STD: [f32; 3] = [0.229, 0.224, 0.225];

/// Standard resolution used across vision backbones (224x224).
pub const DEFAULT_VISION_RESOLUTION: usize = 224;

/// Normalized float tensor representing preprocessed image channels in CHW format.
#[derive(Debug, Clone, PartialEq)]
pub struct ImageTensor {
    pub channels: usize,
    pub height: usize,
    pub width: usize,
    /// Flattened tensor values in Channel-Height-Width (CHW) order.
    pub data: Vec<f32>,
    /// Un-normalized mean color of the image in [0.0, 1.0] [R, G, B].
    pub mean_rgb: [f32; 3],
    /// Overall perceived luminance in [0.0, 1.0].
    pub luminance: f32,
    /// Spatial edge gradient energy density.
    pub edge_energy: f32,
}

impl ImageTensor {
    /// Constructs an `ImageTensor` from raw interleaved RGB pixel bytes.
    /// Resamples to (target_w, target_h) via bilinear interpolation and normalizes with ImageNet statistics.
    pub fn from_rgb_bytes(
        rgb_bytes: &[u8],
        src_width: usize,
        src_height: usize,
        target_width: usize,
        target_height: usize,
    ) -> Result<Self, VisionError> {
        let expected_bytes = src_width * src_height * 3;
        if rgb_bytes.len() < expected_bytes {
            return Err(VisionError::PreprocessingFailed {
                details: format!(
                    "RGB buffer underflow: expected {} bytes for {}x{} image, got {}",
                    expected_bytes,
                    src_width,
                    src_height,
                    rgb_bytes.len()
                ),
            });
        }

        if target_width == 0 || target_height == 0 {
            return Err(VisionError::PreprocessingFailed {
                details: "Target dimensions must be strictly positive".to_string(),
            });
        }

        let channels = 3;
        let mut data = vec![0.0f32; channels * target_height * target_width];
        let mut r_sum = 0.0f32;
        let mut g_sum = 0.0f32;
        let mut b_sum = 0.0f32;
        let total_target_pixels = (target_height * target_width) as f32;

        let scale_x = src_width as f32 / target_width as f32;
        let scale_y = src_height as f32 / target_height as f32;

        // Bilinear interpolation resampling
        for ty in 0..target_height {
            let sy = (ty as f32 + 0.5) * scale_y - 0.5;
            let y0 = sy.floor().max(0.0) as usize;
            let y1 = (y0 + 1).min(src_height - 1);
            let dy = (sy - y0 as f32).clamp(0.0, 1.0);

            for tx in 0..target_width {
                let sx = (tx as f32 + 0.5) * scale_x - 0.5;
                let x0 = sx.floor().max(0.0) as usize;
                let x1 = (x0 + 1).min(src_width - 1);
                let dx = (sx - x0 as f32).clamp(0.0, 1.0);

                // Sample 4 corner pixels in RGB
                let p00_idx = (y0 * src_width + x0) * 3;
                let p10_idx = (y0 * src_width + x1) * 3;
                let p01_idx = (y1 * src_width + x0) * 3;
                let p11_idx = (y1 * src_width + x1) * 3;

                for c in 0..3 {
                    let v00 = rgb_bytes[p00_idx + c] as f32 / 255.0;
                    let v10 = rgb_bytes[p10_idx + c] as f32 / 255.0;
                    let v01 = rgb_bytes[p01_idx + c] as f32 / 255.0;
                    let v11 = rgb_bytes[p11_idx + c] as f32 / 255.0;

                    let top = v00 * (1.0 - dx) + v10 * dx;
                    let bottom = v01 * (1.0 - dx) + v11 * dx;
                    let raw_val = top * (1.0 - dy) + bottom * dy;

                    match c {
                        0 => r_sum += raw_val,
                        1 => g_sum += raw_val,
                        2 => b_sum += raw_val,
                        _ => {}
                    }

                    // Apply ImageNet normalization: (x - mean) / std
                    let normalized = (raw_val - IMAGENET_MEAN[c]) / IMAGENET_STD[c];
                    let chw_idx = c * (target_height * target_width) + ty * target_width + tx;
                    data[chw_idx] = normalized;
                }
            }
        }

        let mean_r = r_sum / total_target_pixels;
        let mean_g = g_sum / total_target_pixels;
        let mean_b = b_sum / total_target_pixels;
        let luminance = 0.299 * mean_r + 0.587 * mean_g + 0.114 * mean_b;

        // Compute edge energy using horizontal & vertical pixel difference on the green channel
        let mut edge_sum = 0.0f32;
        let green_offset = target_height * target_width;
        for y in 0..(target_height - 1) {
            for x in 0..(target_width - 1) {
                let idx = green_offset + y * target_width + x;
                let cur = data[idx];
                let right = data[idx + 1];
                let down = data[idx + target_width];
                let grad_x = right - cur;
                let grad_y = down - cur;
                edge_sum += (grad_x * grad_x + grad_y * grad_y).sqrt();
            }
        }
        let edge_energy = edge_sum / total_target_pixels;

        Ok(Self {
            channels,
            height: target_height,
            width: target_width,
            data,
            mean_rgb: [mean_r, mean_g, mean_b],
            luminance,
            edge_energy,
        })
    }

    /// Returns the raw normalized value at channel c, coordinate (x, y).
    pub fn get(&self, c: usize, x: usize, y: usize) -> Option<f32> {
        if c >= self.channels || x >= self.width || y >= self.height {
            None
        } else {
            let idx = c * (self.height * self.width) + y * self.width + x;
            Some(self.data[idx])
        }
    }
}
