use crate::audio::{GpuType, PerformanceTier};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhisperCompiledBackend {
    Metal,
    Cuda,
    Vulkan,
    HipBlas,
    Cpu,
}

/// Comprehensive acceleration information for diagnostic purposes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccelerationInfo {
    pub compiled_backend: String,
    pub runtime_detected_gpu: String,
    pub use_gpu: bool,
    pub flash_attention_enabled: bool,
    pub performance_tier: String,
    pub diagnostic_summary: String,
}

impl WhisperCompiledBackend {
    pub fn current() -> Self {
        if cfg!(feature = "cuda") {
            Self::Cuda
        } else if cfg!(feature = "vulkan") {
            Self::Vulkan
        } else if cfg!(feature = "hipblas") {
            Self::HipBlas
        } else if cfg!(target_os = "macos") || cfg!(feature = "metal") {
            Self::Metal
        } else {
            Self::Cpu
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Metal => "Metal",
            Self::Cuda => "Cuda",
            Self::Vulkan => "Vulkan",
            Self::HipBlas => "HipBlas",
            Self::Cpu => "Cpu",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WhisperContextAcceleration {
    pub compiled_backend: WhisperCompiledBackend,
    pub runtime_detected_gpu: GpuType,
    pub use_gpu: bool,
    pub flash_attn: bool,
    pub gpu_device: i32,
}

impl WhisperContextAcceleration {
    pub fn status_label(self) -> &'static str {
        match (self.compiled_backend, self.flash_attn) {
            (WhisperCompiledBackend::Metal, true) => "Metal GPU with Flash Attention (Ultra-Fast)",
            (WhisperCompiledBackend::Metal, false) => "Metal GPU acceleration",
            (WhisperCompiledBackend::Cuda, true) => "CUDA GPU with Flash Attention (Ultra-Fast)",
            (WhisperCompiledBackend::Cuda, false) => "CUDA GPU acceleration",
            (WhisperCompiledBackend::Vulkan, _) => "Vulkan GPU acceleration",
            (WhisperCompiledBackend::HipBlas, _) => "HIP BLAS GPU acceleration",
            (WhisperCompiledBackend::Cpu, _) => "CPU processing only",
        }
    }
}

pub fn whisper_context_acceleration_for(
    compiled_backend: WhisperCompiledBackend,
    runtime_detected_gpu: GpuType,
    performance_tier: PerformanceTier,
) -> WhisperContextAcceleration {
    let use_gpu = !matches!(compiled_backend, WhisperCompiledBackend::Cpu);
    let fast_tier = matches!(performance_tier, PerformanceTier::High | PerformanceTier::Ultra);
    let flash_attn = match compiled_backend {
        WhisperCompiledBackend::Metal | WhisperCompiledBackend::Cuda => fast_tier,
        WhisperCompiledBackend::Vulkan | WhisperCompiledBackend::HipBlas | WhisperCompiledBackend::Cpu => false,
    };

    WhisperContextAcceleration {
        compiled_backend,
        runtime_detected_gpu,
        use_gpu,
        flash_attn: use_gpu && flash_attn,
        gpu_device: 0,
    }
}

/// Build a full AccelerationInfo snapshot from current hardware + compiled backend.
pub fn get_acceleration_info() -> AccelerationInfo {
    let compiled_backend = WhisperCompiledBackend::current();
    let hardware_profile = crate::audio::HardwareProfile::detect();
    let acceleration = whisper_context_acceleration_for(
        compiled_backend,
        hardware_profile.gpu_type,
        hardware_profile.performance_tier,
    );

    let gpu_label = match hardware_profile.gpu_type {
        GpuType::None => "None".to_string(),
        GpuType::Metal => "Metal".to_string(),
        GpuType::Cuda => "CUDA".to_string(),
        GpuType::Vulkan => "Vulkan".to_string(),
        GpuType::OpenCL => "OpenCL".to_string(),
    };

    let tier_label = match hardware_profile.performance_tier {
        PerformanceTier::Low => "Low",
        PerformanceTier::Medium => "Medium",
        PerformanceTier::High => "High",
        PerformanceTier::Ultra => "Ultra",
    };

    let diagnostic_summary = if acceleration.use_gpu {
        format!(
            "GPU acceleration ACTIVE — compiled_backend={} runtime_gpu={} flash_attn={} tier={}",
            compiled_backend.as_str(),
            gpu_label,
            acceleration.flash_attn,
            tier_label,
        )
    } else {
        format!(
            "GPU acceleration DISABLED — compiled_backend={} runtime_gpu={} tier={}. \
             To enable CUDA, run with `pnpm run tauri:dev:cuda`.",
            compiled_backend.as_str(),
            gpu_label,
            tier_label,
        )
    };

    AccelerationInfo {
        compiled_backend: compiled_backend.as_str().to_string(),
        runtime_detected_gpu: gpu_label,
        use_gpu: acceleration.use_gpu,
        flash_attention_enabled: acceleration.flash_attn,
        performance_tier: tier_label.to_string(),
        diagnostic_summary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acceleration_vulkan_backend_ignores_runtime_cuda_flash_attention() {
        let params = whisper_context_acceleration_for(
            WhisperCompiledBackend::Vulkan,
            GpuType::Cuda,
            PerformanceTier::High,
        );

        assert_eq!(params.compiled_backend, WhisperCompiledBackend::Vulkan);
        assert_eq!(params.runtime_detected_gpu, GpuType::Cuda);
        assert!(params.use_gpu);
        assert!(!params.flash_attn);
    }

    #[test]
    fn acceleration_vulkan_backend_keeps_gpu_without_runtime_gpu_detection() {
        let params = whisper_context_acceleration_for(
            WhisperCompiledBackend::Vulkan,
            GpuType::None,
            PerformanceTier::Low,
        );

        assert!(params.use_gpu);
        assert!(!params.flash_attn);
    }

    #[test]
    fn acceleration_cuda_backend_enables_flash_attention_for_fast_tiers() {
        let high = whisper_context_acceleration_for(
            WhisperCompiledBackend::Cuda,
            GpuType::Cuda,
            PerformanceTier::High,
        );
        let ultra = whisper_context_acceleration_for(
            WhisperCompiledBackend::Cuda,
            GpuType::Cuda,
            PerformanceTier::Ultra,
        );

        assert!(high.use_gpu);
        assert!(high.flash_attn);
        assert!(ultra.use_gpu);
        assert!(ultra.flash_attn);
    }

    #[test]
    fn acceleration_cpu_backend_disables_gpu_and_flash_attention() {
        for runtime_gpu in [GpuType::None, GpuType::Cuda, GpuType::Vulkan] {
            let params = whisper_context_acceleration_for(
                WhisperCompiledBackend::Cpu,
                runtime_gpu,
                PerformanceTier::Ultra,
            );

            assert!(!params.use_gpu);
            assert!(!params.flash_attn);
        }
    }
}
