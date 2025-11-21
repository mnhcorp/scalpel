use crate::kernel::{FunctionInfo, Symbol};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationPlan {
    pub intent: String,
    pub target: TargetInfo,
    pub implementation: Implementation,
    pub risks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetInfo {
    pub symbol: String,
    pub module: String,
    pub semantic_location: String,
    pub address: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implementation {
    #[serde(rename = "type")]
    pub probe_type: String,
    pub code_snippet: String,
    pub is_mutation: bool,
    pub registers_used: Vec<String>,
}

pub struct PromptBuilder;

impl PromptBuilder {
    pub fn build_analysis_prompt(
        user_intent: &str,
        symbol: &Symbol,
        function_info: Option<&FunctionInfo>,
        disassembly: Option<&[String]>,
    ) -> String {
        let mut prompt = format!(
            r#"You are a Linux kernel instrumentation expert. Analyze this request and generate a precise eBPF probe plan.

USER REQUEST: {}

SYMBOL INFORMATION:
- Function: {}
- Address: {:#x}
- Type: {}
- Module: {}

"#,
            user_intent,
            symbol.name,
            symbol.address,
            symbol.symbol_type,
            symbol.module.as_deref().unwrap_or("kernel")
        );

        if let Some(info) = function_info {
            prompt.push_str(&format!(
                r#"FUNCTION DEBUG INFO:
- Source File: {}
- Line: {}
- Size: {:#x} bytes

"#,
                info.file.as_deref().unwrap_or("unknown"),
                info.line.unwrap_or(0),
                info.size
            ));
        }

        if let Some(asm) = disassembly {
            prompt.push_str("DISASSEMBLY:\n");
            for line in asm.iter().take(20) {
                prompt.push_str(&format!("  {}\n", line));
            }
            prompt.push_str("\n");
        }

        prompt.push_str(
            r#"Generate a JSON operation plan with this EXACT structure:
{
  "intent": "Brief description of what this will do",
  "target": {
    "symbol": "function_name",
    "module": "kernel or module name",
    "semantic_location": "entry|exit|specific offset description",
    "address": 0xffffffff81000000,
    "offset": 0 or specific offset in hex
  },
  "implementation": {
    "type": "kprobe|kretprobe|uprobe|tracepoint",
    "code_snippet": "Complete C code for the BPF program using libbpf",
    "is_mutation": true or false,
    "registers_used": ["RDI", "RSI", "RDX", etc.]
  },
  "risks": [
    "List any safety concerns or potential issues"
  ]
}

IMPORTANT:
1. If the request involves READING data, set is_mutation to false
2. If the request involves MODIFYING/WRITING data, set is_mutation to true
3. For read operations, use bpf_probe_read_kernel() or bpf_probe_read_user()
4. For write operations, explain that bpf_override_return or bpf_probe_write_user is needed
5. Use proper BPF helpers and follow verifier constraints
6. Consider register ABI (x86_64: RDI, RSI, RDX, RCX, R8, R9 for args)
7. Return ONLY valid JSON, no markdown formatting

JSON RESPONSE:
"#,
        );

        prompt
    }

    pub fn build_system_prompt() -> String {
        r#"You are Scalpel, an AI assistant specialized in Linux kernel instrumentation using eBPF.

Your capabilities:
- Analyze kernel functions and identify specific code locations
- Generate safe, verifier-compliant eBPF programs
- Map high-level intent to low-level implementation
- Assess safety risks of kernel modifications

Core principles:
1. Safety First: The BPF verifier is the ultimate authority
2. Precision: Target specific instructions, not whole functions
3. Context Awareness: Consider kernel version, calling conventions, data structures
4. Risk Assessment: Always warn about potential dangers of mutations

When analyzing requests:
- Distinguish between observation (safe) and mutation (dangerous)
- Calculate precise offsets for semantic locations
- Generate production-ready BPF code with proper error handling
- Explain register allocation and data access patterns"#.to_string()
    }
}
