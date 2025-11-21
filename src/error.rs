use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScalpelError {
    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("DWARF parsing error: {0}")]
    DwarfParseError(String),

    #[error("eBPF verification failed: {0}")]
    BpfVerificationFailed(String),

    #[error("eBPF loading failed: {0}")]
    BpfLoadFailed(String),

    #[error("Probe attachment failed: {0}")]
    ProbeAttachmentFailed(String),

    #[error("Unsafe operation rejected: {0}")]
    UnsafeOperationRejected(String),

    #[error("CONFIG_BPF_KPROBE_OVERRIDE not enabled")]
    KprobeOverrideDisabled,

    #[error("LLM API error: {0}")]
    LlmApiError(String),

    #[error("Invalid operation plan: {0}")]
    InvalidOperationPlan(String),

    #[error("Kernel version not supported: {0}")]
    UnsupportedKernelVersion(String),

    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ScalpelError>;
