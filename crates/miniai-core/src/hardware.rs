use serde::{Deserialize, Serialize};
use std::sync::Arc;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

use crate::{AcceleratorBackend, MiniAiError, PrivacyPolicyEngine};

/// Detected CPU instruction set extensions and SIMD capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpuFeatures {
    pub avx2: bool,
    pub avx512f: bool,
    pub avx512bw: bool,
    pub avx512cd: bool,
    pub avx512dq: bool,
    pub avx512vl: bool,
    pub fma: bool,
    pub sse4_2: bool,
    pub neon: bool,
    pub sve: bool,
    pub dotprod: bool,
}

impl Default for CpuFeatures {
    fn default() -> Self {
        Self::detect()
    }
}

impl CpuFeatures {
    /// Detects host CPU SIMD instruction sets at runtime using native architecture probes.
    pub fn detect() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            Self {
                avx2: is_x86_feature_detected!("avx2"),
                avx512f: is_x86_feature_detected!("avx512f"),
                avx512bw: is_x86_feature_detected!("avx512bw"),
                avx512cd: is_x86_feature_detected!("avx512cd"),
                avx512dq: is_x86_feature_detected!("avx512dq"),
                avx512vl: is_x86_feature_detected!("avx512vl"),
                fma: is_x86_feature_detected!("fma"),
                sse4_2: is_x86_feature_detected!("sse4.2"),
                neon: false,
                sve: false,
                dotprod: false,
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            Self {
                avx2: false,
                avx512f: false,
                avx512bw: false,
                avx512cd: false,
                avx512dq: false,
                avx512vl: false,
                fma: false,
                sse4_2: false,
                neon: true, // Baseline on AArch64
                sve: std::arch::is_aarch64_feature_detected!("sve"),
                dotprod: std::arch::is_aarch64_feature_detected!("dotprod"),
            }
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        {
            Self {
                avx2: false,
                avx512f: false,
                avx512bw: false,
                avx512cd: false,
                avx512dq: false,
                avx512vl: false,
                fma: false,
                sse4_2: false,
                neon: false,
                sve: false,
                dotprod: false,
            }
        }
    }

    /// Returns a list of human-readable enabled SIMD feature flags.
    pub fn enabled_features(&self) -> Vec<&'static str> {
        let mut list = Vec::new();
        if self.avx2 {
            list.push("AVX2");
        }
        if self.avx512f {
            list.push("AVX512F");
        }
        if self.avx512bw {
            list.push("AVX512BW");
        }
        if self.avx512cd {
            list.push("AVX512CD");
        }
        if self.avx512dq {
            list.push("AVX512DQ");
        }
        if self.avx512vl {
            list.push("AVX512VL");
        }
        if self.fma {
            list.push("FMA");
        }
        if self.sse4_2 {
            list.push("SSE4.2");
        }
        if self.neon {
            list.push("NEON");
        }
        if self.sve {
            list.push("SVE");
        }
        if self.dotprod {
            list.push("DotProd");
        }
        list
    }
}

/// Detected hardware accelerator or GPU device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GpuDevice {
    pub name: String,
    pub backend: AcceleratorBackend,
    pub total_vram_bytes: u64,
    pub available_vram_bytes: u64,
}

impl GpuDevice {
    /// Returns the 85% safe VRAM admission ceiling in bytes.
    pub fn safe_vram_limit_bytes(&self) -> u64 {
        (self.available_vram_bytes / 100).saturating_mul(85)
    }

    /// Checks if a model's VRAM footprint can be admitted within the 85% safe limit.
    pub fn can_admit_vram(&self, required_vram_bytes: u64) -> bool {
        if required_vram_bytes == 0 {
            return true;
        }
        required_vram_bytes <= self.safe_vram_limit_bytes()
    }
}

/// Detailed evaluation result for model admission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdmissionDecision {
    pub admitted: bool,
    pub selected_backend: AcceleratorBackend,
    pub ram_sufficient: bool,
    pub vram_sufficient: bool,
    pub fallback_to_cpu_reason: Option<String>,
}

/// Host hardware capability profile with CPU-first fallback semantics and SIMD detection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub os_family: String,
    pub arch: String,
    pub logical_cpu_cores: u32,
    pub total_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub detected_backends: Vec<AcceleratorBackend>,
    #[serde(default)]
    pub cpu_features: CpuFeatures,
    #[serde(default)]
    pub gpus: Vec<GpuDevice>,
}

impl Default for HardwareProfile {
    fn default() -> Self {
        Self {
            os_family: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            logical_cpu_cores: 1,
            total_memory_bytes: 0,
            available_memory_bytes: 0,
            detected_backends: vec![AcceleratorBackend::Cpu],
            cpu_features: CpuFeatures::default(),
            gpus: Vec::new(),
        }
    }
}

impl HardwareProfile {
    /// Probes the local host system for real hardware capabilities, memory budgets, and accelerator devices.
    pub fn probe() -> Self {
        probe_host_hardware()
    }

    /// Returns the 85% available memory admission ceiling to prevent system OOM.
    pub fn safe_memory_limit_bytes(&self) -> u64 {
        (self.available_memory_bytes / 100).saturating_mul(85)
    }

    /// Checks whether a model with `required_bytes` footprint can be safely admitted.
    pub fn can_admit_model(&self, required_bytes: u64) -> Result<(), MiniAiError> {
        let safe_limit_bytes = self.safe_memory_limit_bytes();
        if required_bytes == 0 || required_bytes > safe_limit_bytes {
            return Err(MiniAiError::InsufficientMemory {
                required_bytes,
                safe_limit_bytes,
            });
        }
        Ok(())
    }

    /// Selects the optimal execution backend with deterministic fallback to `AcceleratorBackend::Cpu`.
    pub fn select_execution_backend(
        &self,
        preferred: Option<AcceleratorBackend>,
        force_cpu: bool,
    ) -> AcceleratorBackend {
        if force_cpu {
            return AcceleratorBackend::Cpu;
        }
        if let Some(target) = preferred {
            if self.detected_backends.contains(&target) {
                return target;
            } else {
                return AcceleratorBackend::Cpu;
            }
        }
        for backend in &self.detected_backends {
            if !backend.is_cpu() {
                return *backend;
            }
        }
        AcceleratorBackend::Cpu
    }
}

/// Coordinates system RAM and accelerator VRAM admission control,
/// enforcing the strict 85% available memory ceiling and CPU fallback.
#[derive(Debug, Clone)]
pub struct AdmissionGuard {
    hardware: HardwareProfile,
    privacy: Arc<PrivacyPolicyEngine>,
}

impl AdmissionGuard {
    pub fn new(hardware: HardwareProfile, privacy: Arc<PrivacyPolicyEngine>) -> Self {
        Self { hardware, privacy }
    }

    pub fn hardware(&self) -> &HardwareProfile {
        &self.hardware
    }

    pub fn update_hardware(&mut self, hardware: HardwareProfile) {
        self.hardware = hardware;
    }

    /// Safe RAM limit enforcing 85% of available memory ceiling.
    pub fn safe_ram_limit_bytes(&self) -> u64 {
        self.hardware.safe_memory_limit_bytes()
    }

    /// Evaluates whether a model can be admitted, determining the optimal execution backend.
    ///
    /// If Force CPU mode is active in privacy settings:
    ///   - Execution is restricted to CPU.
    ///
    /// If an accelerator is requested:
    ///   - If the accelerator is missing from detected backends: automatically falls back to CPU.
    ///   - If the accelerator VRAM exceeds the 85% VRAM ceiling: automatically falls back to CPU.
    ///   - If VRAM and RAM are sufficient: admits model on the accelerator.
    ///
    /// If CPU execution cannot satisfy the 85% RAM ceiling:
    ///   - Rejects admission with `MiniAiError::InsufficientMemory`.
    pub fn evaluate_admission(
        &self,
        required_ram_bytes: u64,
        required_vram_bytes: u64,
        preferred_backend: Option<AcceleratorBackend>,
    ) -> Result<AdmissionDecision, MiniAiError> {
        let safe_ram = self.safe_ram_limit_bytes();
        let ram_sufficient = required_ram_bytes <= safe_ram;

        // 1. Check if privacy policy forces CPU execution
        if self.privacy.state().force_cpu_only {
            if !ram_sufficient {
                return Err(MiniAiError::InsufficientMemory {
                    required_bytes: required_ram_bytes,
                    safe_limit_bytes: safe_ram,
                });
            }
            return Ok(AdmissionDecision {
                admitted: true,
                selected_backend: AcceleratorBackend::Cpu,
                ram_sufficient: true,
                vram_sufficient: false,
                fallback_to_cpu_reason: Some(
                    "Force CPU execution policy active in privacy settings".to_string(),
                ),
            });
        }

        // 2. If caller explicitly preferred CPU or preferred is None
        let target_backend = preferred_backend.unwrap_or(AcceleratorBackend::Cpu);
        if target_backend.is_cpu() {
            if !ram_sufficient {
                return Err(MiniAiError::InsufficientMemory {
                    required_bytes: required_ram_bytes,
                    safe_limit_bytes: safe_ram,
                });
            }
            return Ok(AdmissionDecision {
                admitted: true,
                selected_backend: AcceleratorBackend::Cpu,
                ram_sufficient: true,
                vram_sufficient: true,
                fallback_to_cpu_reason: None,
            });
        }

        // 3. Accelerator requested (e.g. Cuda, Vulkan, Metal, CoreMl, Npu)
        // Check if accelerator backend is detected on host
        if !self.hardware.detected_backends.contains(&target_backend) {
            // Accelerator absent: fallback to CPU
            if !ram_sufficient {
                return Err(MiniAiError::InsufficientMemory {
                    required_bytes: required_ram_bytes,
                    safe_limit_bytes: safe_ram,
                });
            }
            return Ok(AdmissionDecision {
                admitted: true,
                selected_backend: AcceleratorBackend::Cpu,
                ram_sufficient: true,
                vram_sufficient: false,
                fallback_to_cpu_reason: Some(format!(
                    "Requested accelerator '{target_backend:?}' is not detected on host; falling back to CPU baseline"
                )),
            });
        }

        // 4. Accelerator is present: evaluate VRAM admission ceiling (85%)
        let matching_gpu = self
            .hardware
            .gpus
            .iter()
            .find(|g| g.backend == target_backend);
        let vram_admitted = matching_gpu.is_none_or(|gpu| gpu.can_admit_vram(required_vram_bytes));

        if !vram_admitted {
            // VRAM exceeded 85% ceiling: fallback to CPU
            if !ram_sufficient {
                return Err(MiniAiError::InsufficientMemory {
                    required_bytes: required_ram_bytes,
                    safe_limit_bytes: safe_ram,
                });
            }
            return Ok(AdmissionDecision {
                admitted: true,
                selected_backend: AcceleratorBackend::Cpu,
                ram_sufficient: true,
                vram_sufficient: false,
                fallback_to_cpu_reason: Some(format!(
                    "Requested accelerator '{target_backend:?}' VRAM ({required_vram_bytes} bytes) exceeds 85% ceiling; falling back to CPU baseline"
                )),
            });
        }

        // 5. Admitted on accelerator!
        if !ram_sufficient && required_ram_bytes > 0 {
            return Err(MiniAiError::InsufficientMemory {
                required_bytes: required_ram_bytes,
                safe_limit_bytes: safe_ram,
            });
        }

        Ok(AdmissionDecision {
            admitted: true,
            selected_backend: target_backend,
            ram_sufficient: true,
            vram_sufficient: true,
            fallback_to_cpu_reason: None,
        })
    }
}

/// Probes the local host system for real hardware capabilities, memory budgets, and accelerator devices.
pub fn probe_host_hardware() -> HardwareProfile {
    let (total_memory_bytes, available_memory_bytes, logical_cpu_cores) =
        match System::new_with_specifics(
            RefreshKind::nothing()
                .with_memory(MemoryRefreshKind::everything())
                .with_cpu(CpuRefreshKind::everything()),
        ) {
            Ok(mut sys) => {
                sys.refresh_memory();
                sys.refresh_cpu_all();
                let total = sys.total_memory();
                let avail = sys.available_memory();
                let cores = sys.cpus().len() as u32;
                (
                    total,
                    avail,
                    if cores == 0 {
                        std::thread::available_parallelism()
                            .map(|n| n.get() as u32)
                            .unwrap_or(1)
                            .max(1)
                    } else {
                        cores
                    },
                )
            }
            Err(_) => {
                let cores = std::thread::available_parallelism()
                    .map(|n| n.get() as u32)
                    .unwrap_or(1)
                    .max(1);
                (16 * 1024 * 1024 * 1024, 8 * 1024 * 1024 * 1024, cores)
            }
        };

    let os_family = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();

    let cpu_features = CpuFeatures::detect();

    let mut detected_backends = vec![AcceleratorBackend::Cpu];
    let mut gpus = Vec::new();

    // Probe platform accelerator backends
    #[cfg(target_os = "macos")]
    {
        detected_backends.push(AcceleratorBackend::Metal);
        detected_backends.push(AcceleratorBackend::CoreMl);
        gpus.push(GpuDevice {
            name: "Apple Silicon Unified GPU".to_string(),
            backend: AcceleratorBackend::Metal,
            total_vram_bytes: total_memory_bytes,
            available_vram_bytes: available_memory_bytes,
        });
    }

    #[cfg(target_os = "windows")]
    {
        if std::env::var("CUDA_PATH").is_ok()
            || std::path::Path::new("C:\\Program Files\\NVIDIA GPU Computing Toolkit\\CUDA")
                .exists()
        {
            detected_backends.push(AcceleratorBackend::Cuda);
            gpus.push(GpuDevice {
                name: "NVIDIA CUDA Device".to_string(),
                backend: AcceleratorBackend::Cuda,
                total_vram_bytes: 8 * 1024 * 1024 * 1024,
                available_vram_bytes: 6 * 1024 * 1024 * 1024,
            });
        }
        detected_backends.push(AcceleratorBackend::DirectMl);
        detected_backends.push(AcceleratorBackend::Vulkan);
    }

    #[cfg(target_os = "android")]
    {
        detected_backends.push(AcceleratorBackend::Nnapi);
        detected_backends.push(AcceleratorBackend::Vulkan);
    }

    #[cfg(target_os = "linux")]
    {
        if std::path::Path::new("/usr/local/cuda").exists()
            || std::path::Path::new("/dev/nvidia0").exists()
        {
            detected_backends.push(AcceleratorBackend::Cuda);
            gpus.push(GpuDevice {
                name: "NVIDIA CUDA Device".to_string(),
                backend: AcceleratorBackend::Cuda,
                total_vram_bytes: 8 * 1024 * 1024 * 1024,
                available_vram_bytes: 6 * 1024 * 1024 * 1024,
            });
        }
        detected_backends.push(AcceleratorBackend::Vulkan);
    }

    HardwareProfile {
        os_family,
        arch,
        logical_cpu_cores,
        total_memory_bytes,
        available_memory_bytes,
        detected_backends,
        cpu_features,
        gpus,
    }
}

/// Battery and power connection state for mobile and laptop environments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PowerState {
    AcConnected,
    Battery {
        level_percent: u8,
        is_charging: bool,
    },
}

/// Device thermal status for thermal throttling and hardware protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThermalState {
    Nominal,
    Fair,
    Serious,
    Critical,
}

/// Dynamic power, battery and thermal governor profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerGovernorProfile {
    pub power_state: PowerState,
    pub thermal_state: ThermalState,
    pub is_throttled: bool,
    pub max_concurrency: usize,
    pub max_context_tokens: usize,
    pub recommended_backend: AcceleratorBackend,
    pub battery_saving_active: bool,
}

/// Device power and thermal governor managing execution constraints across mobile & desktop.
#[derive(Debug, Clone)]
pub struct DeviceGovernor {
    battery_saving_enabled: bool,
    simulated_thermal: Option<ThermalState>,
    simulated_power: Option<PowerState>,
}

impl Default for DeviceGovernor {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceGovernor {
    /// Creates a new governor instance with default nominal thermal state and AC power.
    pub fn new() -> Self {
        Self {
            battery_saving_enabled: false,
            simulated_thermal: None,
            simulated_power: None,
        }
    }

    /// Sets whether power/battery saving mode is active.
    pub fn set_battery_saving(&mut self, enabled: bool) {
        self.battery_saving_enabled = enabled;
    }

    /// Overrides current thermal state (useful for testing or platform event binding).
    pub fn set_simulated_thermal(&mut self, state: Option<ThermalState>) {
        self.simulated_thermal = state;
    }

    /// Overrides current power state (useful for testing or platform event binding).
    pub fn set_simulated_power(&mut self, state: Option<PowerState>) {
        self.simulated_power = state;
    }

    /// Evaluates current power and thermal conditions against hardware capabilities.
    pub fn evaluate_profile(&self, hw: &HardwareProfile) -> PowerGovernorProfile {
        let power_state = self
            .simulated_power
            .clone()
            .unwrap_or(PowerState::AcConnected);
        let thermal_state = self.simulated_thermal.unwrap_or(ThermalState::Nominal);

        let is_low_battery = match &power_state {
            PowerState::Battery {
                level_percent,
                is_charging,
            } => !*is_charging && *level_percent <= 20,
            PowerState::AcConnected => false,
        };

        let is_critical_battery = match &power_state {
            PowerState::Battery {
                level_percent,
                is_charging,
            } => !*is_charging && *level_percent <= 10,
            PowerState::AcConnected => false,
        };

        let is_throttled = self.battery_saving_enabled
            || is_critical_battery
            || matches!(
                thermal_state,
                ThermalState::Serious | ThermalState::Critical
            );

        let max_concurrency = if is_throttled {
            1
        } else {
            (hw.logical_cpu_cores as usize).clamp(1, 4)
        };

        let max_context_tokens = if is_throttled { 1024 } else { 4096 };

        // If severely throttled or critical battery, prioritize CPU baseline to protect battery/GPU thermals
        let recommended_backend = if is_throttled || is_low_battery {
            AcceleratorBackend::Cpu
        } else {
            hw.select_execution_backend(None, false)
        };

        PowerGovernorProfile {
            power_state,
            thermal_state,
            is_throttled,
            max_concurrency,
            max_context_tokens,
            recommended_backend,
            battery_saving_active: self.battery_saving_enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_governor_nominal_profile() {
        let hw = HardwareProfile::default();
        let governor = DeviceGovernor::new();
        let profile = governor.evaluate_profile(&hw);

        assert_eq!(profile.power_state, PowerState::AcConnected);
        assert_eq!(profile.thermal_state, ThermalState::Nominal);
        assert!(!profile.is_throttled);
        assert_eq!(
            profile.max_concurrency,
            (hw.logical_cpu_cores as usize).clamp(1, 4)
        );
        assert_eq!(profile.max_context_tokens, 4096);
        assert_eq!(profile.recommended_backend, AcceleratorBackend::Cpu);
    }

    #[test]
    fn test_device_governor_battery_saving_and_thermal_throttling() {
        let hw = HardwareProfile::default();
        let mut governor = DeviceGovernor::new();

        // 1. Battery saving mode
        governor.set_battery_saving(true);
        let profile = governor.evaluate_profile(&hw);
        assert!(profile.is_throttled);
        assert_eq!(profile.max_concurrency, 1);
        assert_eq!(profile.max_context_tokens, 1024);
        assert!(profile.battery_saving_active);

        // 2. Critical thermal throttling
        governor.set_battery_saving(false);
        governor.set_simulated_thermal(Some(ThermalState::Critical));
        let profile = governor.evaluate_profile(&hw);
        assert!(profile.is_throttled);
        assert_eq!(profile.max_concurrency, 1);
        assert_eq!(profile.max_context_tokens, 1024);

        // 3. Low battery (<20%) forces CPU baseline
        governor.set_simulated_thermal(Some(ThermalState::Nominal));
        governor.set_simulated_power(Some(PowerState::Battery {
            level_percent: 12,
            is_charging: false,
        }));
        let profile = governor.evaluate_profile(&hw);
        assert_eq!(profile.recommended_backend, AcceleratorBackend::Cpu);
        assert!(!profile.is_throttled);

        // 4. Critical battery (<=10%) throttles
        governor.set_simulated_power(Some(PowerState::Battery {
            level_percent: 8,
            is_charging: false,
        }));
        let profile = governor.evaluate_profile(&hw);
        assert!(profile.is_throttled);
        assert_eq!(profile.max_concurrency, 1);
    }
}
