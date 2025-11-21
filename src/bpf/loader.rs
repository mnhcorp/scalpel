use crate::error::{Result, ScalpelError};
use crate::kernel::{Probe, ProbeType};
use std::path::Path;
use std::process::Command;

pub struct BpfProgram {
    pub code: String,
    pub compiled: bool,
}

impl BpfProgram {
    pub fn new(code: String) -> Self {
        Self {
            code,
            compiled: false,
        }
    }
}

pub struct BpfLoader;

impl BpfLoader {
    /// Compile BPF code (in a real implementation, this would use clang)
    pub fn compile(code: &str) -> Result<Vec<u8>> {
        tracing::info!("Compiling BPF program...");

        // In a real implementation, we would:
        // 1. Write code to a temporary .c file
        // 2. Invoke clang with BPF target
        // 3. Load the resulting .o file
        //
        // For now, simulate successful compilation
        tracing::warn!("BPF compilation simulated (not actually compiling)");

        // Check if code looks valid
        if !code.contains("SEC(") {
            return Err(ScalpelError::BpfVerificationFailed(
                "Missing SEC() declaration".to_string()
            ));
        }

        if !code.contains("LICENSE") {
            return Err(ScalpelError::BpfVerificationFailed(
                "Missing GPL license declaration".to_string()
            ));
        }

        // Return mock bytecode
        Ok(vec![0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
    }

    /// Verify BPF bytecode (in a real implementation, this would use the kernel verifier)
    pub fn verify(bytecode: &[u8]) -> Result<()> {
        tracing::info!("Verifying BPF program with kernel verifier...");

        // In a real implementation, we would:
        // 1. Attempt to load the program with BPF_PROG_LOAD
        // 2. The kernel verifier would check:
        //    - Bounded loops
        //    - Valid memory accesses
        //    - Proper helper usage
        //    - Stack limits
        //    - Instruction count
        //
        // For now, simulate successful verification
        tracing::warn!("BPF verification simulated (not actually verifying)");

        if bytecode.is_empty() {
            return Err(ScalpelError::BpfVerificationFailed(
                "Empty bytecode".to_string()
            ));
        }

        Ok(())
    }

    /// Load and attach BPF program
    pub fn load_and_attach(
        bytecode: &[u8],
        probe_type: ProbeType,
        target: &str,
        offset: Option<u64>,
    ) -> Result<Probe> {
        Self::verify(bytecode)?;

        tracing::info!("Loading BPF program...");
        tracing::info!("Attaching {:?} to {} (offset: {:?})", probe_type, target, offset);

        // In a real implementation, we would:
        // 1. Use bpf() syscall with BPF_PROG_LOAD
        // 2. Use perf_event_open() or bpf_link for attachment
        // 3. Return the actual probe FD
        //
        // For now, simulate successful attachment
        tracing::warn!("BPF loading simulated (not actually loading)");

        let probe = crate::kernel::probe::register_probe(
            probe_type,
            target.to_string(),
            offset,
            false, // is_mutation
        )?;

        tracing::info!("Probe {} attached successfully", probe.id);
        Ok(probe)
    }

    /// Check if the system supports required BPF features
    pub fn check_system_capabilities() -> Result<SystemCapabilities> {
        tracing::info!("Checking system BPF capabilities...");

        let mut caps = SystemCapabilities::default();

        // Check for CONFIG_BPF
        caps.bpf_enabled = Self::check_kernel_config("CONFIG_BPF");

        // Check for CONFIG_BPF_KPROBE_OVERRIDE
        caps.kprobe_override = Self::check_kernel_config("CONFIG_BPF_KPROBE_OVERRIDE");

        // Check for debugfs
        caps.debugfs_available = Path::new("/sys/kernel/debug").exists();

        // Check for tracefs
        caps.tracefs_available = Path::new("/sys/kernel/tracing").exists()
            || Path::new("/sys/kernel/debug/tracing").exists();

        // Check permissions (CAP_BPF or root)
        caps.has_permissions = std::env::var("USER").unwrap_or_default() == "root";

        Ok(caps)
    }

    fn check_kernel_config(config: &str) -> bool {
        // Try to read from /proc/config.gz or /boot/config-*
        if let Ok(output) = Command::new("zcat")
            .arg("/proc/config.gz")
            .output()
        {
            let content = String::from_utf8_lossy(&output.stdout);
            return content.contains(&format!("{}=", config));
        }

        // Try /boot/config
        if let Ok(output) = Command::new("uname")
            .arg("-r")
            .output()
        {
            let kernel = String::from_utf8_lossy(&output.stdout);
            let config_path = format!("/boot/config-{}", kernel.trim());
            if let Ok(content) = std::fs::read_to_string(config_path) {
                return content.contains(&format!("{}=", config));
            }
        }

        // Assume available if we can't check
        true
    }
}

#[derive(Debug, Default)]
pub struct SystemCapabilities {
    pub bpf_enabled: bool,
    pub kprobe_override: bool,
    pub debugfs_available: bool,
    pub tracefs_available: bool,
    pub has_permissions: bool,
}

impl SystemCapabilities {
    pub fn is_ready(&self) -> bool {
        self.bpf_enabled && self.has_permissions
    }

    pub fn missing_capabilities(&self) -> Vec<String> {
        let mut missing = Vec::new();

        if !self.bpf_enabled {
            missing.push("CONFIG_BPF kernel support".to_string());
        }
        if !self.has_permissions {
            missing.push("Root or CAP_BPF permissions".to_string());
        }
        if !self.debugfs_available && !self.tracefs_available {
            missing.push("debugfs or tracefs mount".to_string());
        }

        missing
    }
}
