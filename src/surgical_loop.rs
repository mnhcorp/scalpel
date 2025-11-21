use crate::bpf::{BpfCodeGenerator, BpfLoader};
use crate::config::Config;
use crate::error::{Result, ScalpelError};
use crate::kernel::{DwarfAnalyzer, Probe, ProbeType, SymbolResolver};
use crate::llm::{ClaudeClient, OperationPlan, PromptBuilder};
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SurgicalPhase {
    Diagnostic,
    Prescription,
    Procedure,
    Monitor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurgicalResult {
    pub phase: SurgicalPhase,
    pub plan: Option<OperationPlan>,
    pub probe: Option<Probe>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

pub struct SurgicalLoop {
    config: Config,
    symbol_resolver: SymbolResolver,
    dwarf_analyzer: DwarfAnalyzer,
    llm_client: Option<ClaudeClient>,
}

impl SurgicalLoop {
    pub fn new(config: Config) -> Self {
        let symbol_resolver = SymbolResolver::new()
            .unwrap_or_else(|_| {
                tracing::warn!("Failed to load symbols, creating empty resolver");
                SymbolResolver::empty()
            });

        let dwarf_analyzer = DwarfAnalyzer::new(config.vmlinux_path.clone());

        let llm_client = if config.api_key.is_some() {
            ClaudeClient::new(config.clone()).ok()
        } else {
            None
        };

        Self {
            config,
            symbol_resolver,
            dwarf_analyzer,
            llm_client,
        }
    }

    /// Main entry point: Run all 4 phases of the surgical loop
    pub async fn analyze(&mut self, user_prompt: &str) -> Result<SurgicalResult> {
        println!("{}", "╔═══════════════════════════════════════════════════════════╗".cyan());
        println!("{}", "║  SCALPEL - Surgical Loop Initiated                       ║".cyan());
        println!("{}", "╚═══════════════════════════════════════════════════════════╝".cyan());
        println!();

        let mut result = SurgicalResult {
            phase: SurgicalPhase::Diagnostic,
            plan: None,
            probe: None,
            warnings: Vec::new(),
            errors: Vec::new(),
        };

        // Phase 1: The Diagnostic (Analysis)
        println!("{}", "Phase 1: The Diagnostic".bold().green());
        println!("{}", "═".repeat(60).green());

        let plan = match self.phase1_diagnostic(user_prompt).await {
            Ok(plan) => {
                println!("✓ Analysis complete");
                plan
            }
            Err(e) => {
                result.errors.push(format!("Diagnostic failed: {}", e));
                return Ok(result);
            }
        };

        result.plan = Some(plan.clone());
        println!();

        // Phase 2: The Prescription (Code Gen)
        result.phase = SurgicalPhase::Prescription;
        println!("{}", "Phase 2: The Prescription".bold().green());
        println!("{}", "═".repeat(60).green());

        let code = match self.phase2_prescription(&plan) {
            Ok(code) => {
                println!("✓ BPF code generated");
                code
            }
            Err(e) => {
                result.errors.push(format!("Prescription failed: {}", e));
                return Ok(result);
            }
        };
        println!();

        // Safety check before Phase 3
        if plan.implementation.is_mutation {
            result.warnings.push("This operation will MODIFY kernel state".to_string());

            if self.config.require_confirmation {
                println!("{}", "⚠ WARNING: MUTATION OPERATION".bold().red());
                println!("{}", "This operation will modify kernel state!".red());
                println!("Risks:");
                for risk in &plan.risks {
                    println!("  - {}", risk.red());
                }
                println!();
                println!("In interactive mode, you would be prompted for confirmation.");
                result.warnings.push("Skipping execution due to mutation safety".to_string());
                return Ok(result);
            }
        }

        // Phase 3: The Procedure (Injection)
        result.phase = SurgicalPhase::Procedure;
        println!("{}", "Phase 3: The Procedure".bold().green());
        println!("{}", "═".repeat(60).green());

        let probe = match self.phase3_procedure(&plan, &code) {
            Ok(probe) => {
                println!("✓ Probe attached: {}", probe.id);
                probe
            }
            Err(e) => {
                result.errors.push(format!("Procedure failed: {}", e));
                return Ok(result);
            }
        };

        result.probe = Some(probe);
        println!();

        // Phase 4: The Monitor (Telemetry)
        result.phase = SurgicalPhase::Monitor;
        println!("{}", "Phase 4: The Monitor".bold().green());
        println!("{}", "═".repeat(60).green());
        println!("✓ Monitoring active (use interactive mode for live telemetry)");
        println!();

        println!("{}", "Surgical loop complete!".bold().green());
        Ok(result)
    }

    /// Phase 1: Symbol resolution, DWARF parsing, semantic mapping
    async fn phase1_diagnostic(&mut self, user_prompt: &str) -> Result<OperationPlan> {
        println!("Analyzing prompt: {}", user_prompt.italic());

        // Extract function name from prompt (simple heuristic)
        let function_name = self.extract_function_name(user_prompt)?;
        println!("Target function: {}", function_name.bold());

        // Resolve symbol
        let symbol = self.symbol_resolver.resolve(&function_name)?;
        println!("Symbol address: {:#x}", symbol.address);

        // Get DWARF info
        let function_info = self.dwarf_analyzer.get_function_info(&function_name).ok();

        // Get disassembly
        let disassembly = if let Ok(asm) = self.dwarf_analyzer.get_disassembly(symbol.address, 256) {
            Some(asm)
        } else {
            None
        };

        // Call LLM for analysis
        let llm_client = self.llm_client.as_ref()
            .ok_or_else(|| ScalpelError::ConfigError("LLM client not initialized".to_string()))?;

        let prompt = PromptBuilder::build_analysis_prompt(
            user_prompt,
            symbol,
            function_info.as_ref(),
            disassembly.as_deref(),
        );

        let system_prompt = PromptBuilder::build_system_prompt();

        println!("Consulting Claude for surgical plan...");
        let response = llm_client.analyze(&prompt, Some(system_prompt)).await?;

        // Parse JSON response
        let plan: OperationPlan = serde_json::from_str(&response)
            .map_err(|e| ScalpelError::InvalidOperationPlan(format!("Failed to parse LLM response: {}", e)))?;

        Ok(plan)
    }

    /// Phase 2: Generate BPF code
    fn phase2_prescription(&self, plan: &OperationPlan) -> Result<String> {
        println!("Generating BPF code for: {}", plan.intent.italic());
        println!("Probe type: {}", plan.implementation.probe_type);
        println!("Mutation: {}", if plan.implementation.is_mutation { "YES".red() } else { "NO".green() });

        let code = BpfCodeGenerator::generate_from_plan(plan)?;
        println!("Generated {} lines of code", code.lines().count());

        Ok(code)
    }

    /// Phase 3: Compile, verify, and attach probe
    fn phase3_procedure(&self, plan: &OperationPlan, code: &str) -> Result<Probe> {
        println!("Compiling BPF program...");
        let bytecode = BpfLoader::compile(code)?;

        println!("Verifying with BPF verifier...");
        BpfLoader::verify(&bytecode)?;

        if plan.implementation.is_mutation && !self.config.safety_guards {
            return Err(ScalpelError::UnsafeOperationRejected(
                "Safety guards are disabled but mutation operation requested".to_string()
            ));
        }

        let probe_type = match plan.implementation.probe_type.as_str() {
            "kprobe" => ProbeType::Kprobe,
            "kretprobe" => ProbeType::Kretprobe,
            "uprobe" => ProbeType::Uprobe,
            "uretprobe" => ProbeType::Uretprobe,
            "tracepoint" => ProbeType::Tracepoint,
            _ => ProbeType::Kprobe,
        };

        println!("Attaching probe...");
        let probe = BpfLoader::load_and_attach(
            &bytecode,
            probe_type,
            &plan.target.symbol,
            plan.target.offset,
        )?;

        Ok(probe)
    }

    /// Extract function name from natural language prompt
    fn extract_function_name(&self, prompt: &str) -> Result<String> {
        // Simple heuristic: look for common function patterns
        let words: Vec<&str> = prompt.split_whitespace().collect();

        for word in words {
            // Check if word matches kernel function naming convention
            if word.contains('_') && !word.starts_with('-') {
                let clean = word.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
                if self.symbol_resolver.resolve(clean).is_ok() {
                    return Ok(clean.to_string());
                }
            }
        }

        // If no function found, try searching for partial matches
        for word in prompt.split_whitespace() {
            let matches = self.symbol_resolver.search(word);
            if !matches.is_empty() {
                return Ok(matches[0].name.clone());
            }
        }

        Err(ScalpelError::SymbolNotFound(
            "Could not extract function name from prompt".to_string()
        ))
    }
}
