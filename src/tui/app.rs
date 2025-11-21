use crate::config::Config;
use crate::error::Result;
use crate::kernel::{KernelVitals, Probe};
use crate::surgical_loop::SurgicalLoop;
use chrono::{DateTime, Utc};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Focus {
    Vitals,
    Disassembly,
    Probes,
    Logs,
}

pub struct App {
    pub config: Config,
    pub input: String,
    pub input_mode: bool,
    pub focus: Focus,
    pub vitals: Option<KernelVitals>,
    pub last_vitals_update: Instant,
    pub probes: Vec<Probe>,
    pub logs: Vec<LogEntry>,
    pub disassembly: Vec<String>,
    pub log_scroll: usize,
    pub surgical_loop: SurgicalLoop,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Success,
}

impl App {
    pub fn new(config: Config) -> Self {
        let surgical_loop = SurgicalLoop::new(config.clone());

        let mut app = Self {
            config,
            input: String::new(),
            input_mode: false,
            focus: Focus::Logs,
            vitals: None,
            last_vitals_update: Instant::now(),
            probes: Vec::new(),
            logs: Vec::new(),
            disassembly: Vec::new(),
            log_scroll: 0,
            surgical_loop,
        };

        app.add_log(LogLevel::Info, "Scalpel v1.0.0 - Kernel Surgeon Edition");
        app.add_log(LogLevel::Info, "Press 'i' to enter command mode, 'q' to quit, Tab to switch focus");
        app.add_log(LogLevel::Info, "Type your instrumentation request in natural language");

        app
    }

    pub fn add_log(&mut self, level: LogLevel, message: impl Into<String>) {
        self.logs.push(LogEntry {
            timestamp: Utc::now(),
            level,
            message: message.into(),
        });

        // Keep only last 1000 entries
        if self.logs.len() > 1000 {
            self.logs.drain(0..100);
        }
    }

    pub fn update_vitals(&mut self) {
        // Update every 2 seconds
        if self.last_vitals_update.elapsed() > Duration::from_secs(2) {
            if let Ok(vitals) = KernelVitals::collect() {
                self.vitals = Some(vitals);
            }
            self.last_vitals_update = Instant::now();
        }
    }

    pub async fn handle_command(&mut self, command: String) -> Result<()> {
        if command.trim().is_empty() {
            return Ok(());
        }

        self.add_log(LogLevel::Info, format!("> {}", command));

        // Handle built-in commands
        match command.trim() {
            "help" => {
                self.add_log(LogLevel::Info, "Available commands:");
                self.add_log(LogLevel::Info, "  help - Show this help");
                self.add_log(LogLevel::Info, "  list - List active probes");
                self.add_log(LogLevel::Info, "  detach <id> - Detach a probe");
                self.add_log(LogLevel::Info, "  clear - Clear logs");
                self.add_log(LogLevel::Info, "  Or type a natural language instrumentation request");
                return Ok(());
            }
            "list" => {
                if let Ok(probes) = crate::kernel::probe::list_active_probes() {
                    if probes.is_empty() {
                        self.add_log(LogLevel::Info, "No active probes");
                    } else {
                        for probe in probes {
                            self.add_log(LogLevel::Info, format!("{}", probe));
                        }
                    }
                }
                return Ok(());
            }
            "clear" => {
                self.logs.clear();
                self.add_log(LogLevel::Success, "Logs cleared");
                return Ok(());
            }
            cmd if cmd.starts_with("detach ") => {
                let probe_id = cmd.strip_prefix("detach ").unwrap().trim();
                match crate::kernel::probe::detach_probe(probe_id) {
                    Ok(_) => {
                        self.add_log(LogLevel::Success, format!("Probe {} detached", probe_id));
                        self.refresh_probes();
                    }
                    Err(e) => {
                        self.add_log(LogLevel::Error, format!("Failed to detach: {}", e));
                    }
                }
                return Ok(());
            }
            _ => {}
        }

        // Handle as natural language prompt
        self.add_log(LogLevel::Info, "Analyzing request...");

        match self.surgical_loop.analyze(&command).await {
            Ok(result) => {
                if let Some(plan) = &result.plan {
                    self.add_log(LogLevel::Success, format!("Intent: {}", plan.intent));
                    self.add_log(LogLevel::Info, format!("Target: {}", plan.target.symbol));

                    if plan.implementation.is_mutation {
                        self.add_log(LogLevel::Warning, "This operation would modify kernel state");
                        for risk in &plan.risks {
                            self.add_log(LogLevel::Warning, format!("Risk: {}", risk));
                        }
                    }
                }

                if let Some(probe) = result.probe {
                    self.add_log(LogLevel::Success, format!("Probe attached: {}", probe.id));
                    self.probes.push(probe);
                }

                for warning in result.warnings {
                    self.add_log(LogLevel::Warning, warning);
                }

                for error in result.errors {
                    self.add_log(LogLevel::Error, error);
                }
            }
            Err(e) => {
                self.add_log(LogLevel::Error, format!("Failed: {}", e));
            }
        }

        self.refresh_probes();
        Ok(())
    }

    pub fn refresh_probes(&mut self) {
        if let Ok(probes) = crate::kernel::probe::list_active_probes() {
            self.probes = probes;
        }
    }

    pub fn scroll_logs_up(&mut self) {
        if self.log_scroll > 0 {
            self.log_scroll -= 1;
        }
    }

    pub fn scroll_logs_down(&mut self) {
        if self.log_scroll < self.logs.len().saturating_sub(1) {
            self.log_scroll += 1;
        }
    }

    pub fn next_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Vitals => Focus::Disassembly,
            Focus::Disassembly => Focus::Probes,
            Focus::Probes => Focus::Logs,
            Focus::Logs => Focus::Vitals,
        };
    }
}
