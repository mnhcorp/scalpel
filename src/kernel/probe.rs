use crate::error::{Result, ScalpelError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProbeType {
    Kprobe,
    Kretprobe,
    Uprobe,
    Uretprobe,
    Tracepoint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProbeStatus {
    Active,
    Failed,
    Detached,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub id: String,
    pub probe_type: ProbeType,
    pub target: String,
    pub offset: Option<u64>,
    pub status: ProbeStatus,
    pub is_mutation: bool,
    pub created_at: DateTime<Utc>,
    pub event_count: u64,
}

impl fmt::Display for Probe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let probe_type = match self.probe_type {
            ProbeType::Kprobe => "kprobe",
            ProbeType::Kretprobe => "kretprobe",
            ProbeType::Uprobe => "uprobe",
            ProbeType::Uretprobe => "uretprobe",
            ProbeType::Tracepoint => "tracepoint",
        };

        let status = match self.status {
            ProbeStatus::Active => "ACTIVE",
            ProbeStatus::Failed => "FAILED",
            ProbeStatus::Detached => "DETACHED",
        };

        let offset_str = if let Some(offset) = self.offset {
            format!("+{:#x}", offset)
        } else {
            String::new()
        };

        let mutation_flag = if self.is_mutation { " [MUTATION]" } else { "" };

        write!(
            f,
            "{} {}:{}{} [{}] events={} {}",
            self.id,
            probe_type,
            self.target,
            offset_str,
            status,
            self.event_count,
            mutation_flag
        )
    }
}

lazy_static::lazy_static! {
    static ref PROBE_REGISTRY: Arc<Mutex<HashMap<String, Probe>>> = Arc::new(Mutex::new(HashMap::new()));
}

pub fn register_probe(
    probe_type: ProbeType,
    target: String,
    offset: Option<u64>,
    is_mutation: bool,
) -> Result<Probe> {
    let probe = Probe {
        id: Uuid::new_v4().to_string()[..8].to_string(),
        probe_type,
        target,
        offset,
        status: ProbeStatus::Active,
        is_mutation,
        created_at: Utc::now(),
        event_count: 0,
    };

    let mut registry = PROBE_REGISTRY.lock().unwrap();
    registry.insert(probe.id.clone(), probe.clone());

    tracing::info!("Registered probe: {}", probe);
    Ok(probe)
}

pub fn list_active_probes() -> Result<Vec<Probe>> {
    let registry = PROBE_REGISTRY.lock().unwrap();
    Ok(registry
        .values()
        .filter(|p| p.status == ProbeStatus::Active)
        .cloned()
        .collect())
}

pub fn detach_probe(probe_id: &str) -> Result<()> {
    let mut registry = PROBE_REGISTRY.lock().unwrap();

    if let Some(probe) = registry.get_mut(probe_id) {
        probe.status = ProbeStatus::Detached;
        tracing::info!("Detached probe: {}", probe_id);
        Ok(())
    } else {
        Err(ScalpelError::InvalidOperationPlan(
            format!("Probe {} not found", probe_id)
        ))
    }
}

pub fn update_event_count(probe_id: &str, count: u64) -> Result<()> {
    let mut registry = PROBE_REGISTRY.lock().unwrap();

    if let Some(probe) = registry.get_mut(probe_id) {
        probe.event_count += count;
        Ok(())
    } else {
        Err(ScalpelError::InvalidOperationPlan(
            format!("Probe {} not found", probe_id)
        ))
    }
}
