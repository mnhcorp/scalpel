# Scalpel 🔬

**Surgical Precision for the Linux Kernel**

> Version 1.0.0 - Kernel Surgeon Edition

Scalpel is a terminal-native, agentic CLI that brings surgical precision to Linux kernel observability and modification. It bridges the gap between high-level operational intent ("Fix the packet drop in TCP") and low-level implementation (calculating hex offsets, writing eBPF bytecode).

```
╔═══════════════════════════════════════════════════════════╗
║  SCALPEL v1.0.0 - Kernel Surgeon Edition                 ║
║  Surgical Precision for the Linux Kernel                  ║
╚═══════════════════════════════════════════════════════════╝
```

## 🎯 Core Philosophy

1. **No Recompilation**: All instrumentation is dynamic (eBPF/kprobe/uprobe)
2. **Safety First**: The BPF Verifier is the ultimate gatekeeper
3. **Semantic Addressing**: Users address logic ("the checksum error block"), not memory addresses
4. **Zero Overhead**: Hooks are JIT-compiled and only incur cost when active

## ✨ Features

- 🤖 **AI-Powered Analysis**: Natural language kernel instrumentation using Claude 3.5 Sonnet
- 🔍 **Dynamic Instrumentation**: Insert probes without recompiling the kernel
- 📊 **Real-time TUI**: Beautiful terminal interface inspired by Claude Code
- 🛡️ **Safety Guards**: Multiple layers of protection for kernel modifications
- 🎯 **Semantic Targeting**: Find code by intent, not just by address
- 📈 **Live Telemetry**: Stream events from eBPF ring buffers in real-time

## 🏗️ Architecture: The Surgical Loop

Scalpel operates through a 4-phase state machine:

### Phase 1: The Diagnostic (Analysis)
- Symbol resolution via `/proc/kallsyms`
- DWARF parsing for debug information
- Semantic mapping using AI to find specific code locations

### Phase 2: The Prescription (Code Generation)
- AI-generated eBPF/C code with proper kernel headers
- Automatic register mapping for function arguments
- Context-aware helper selection

### Phase 3: The Procedure (Injection)
- BPF verification and compilation
- Kprobe/uprobe attachment at calculated offsets
- Safety confirmations for mutation operations

### Phase 4: The Monitor (Telemetry)
- Real-time event streaming from BPF ring buffers
- Human-readable formatting of kernel data structures
- Live metrics and probe status

## 📦 Installation

### Prerequisites

- Linux kernel 5.8+ with eBPF support (`CONFIG_BPF=y`)
- Rust 1.70+ (for building from source)
- Root or `CAP_BPF` capability
- Anthropic API key for Claude

### Install from Source

```bash
# Clone the repository
git clone https://github.com/mnhcorp/scalpel.git
cd scalpel

# Build release binary
cargo build --release

# Install to /usr/local/bin
sudo cp target/release/scalpel /usr/local/bin/

# Or install via cargo
cargo install --path .
```

### Configuration

Create a configuration file at `~/.config/scalpel/config.toml`:

```toml
# Anthropic API key (or set ANTHROPIC_API_KEY env var)
api_key = "sk-ant-..."

# Claude model to use
model = "claude-3-5-sonnet-20241022"

# API endpoint
endpoint = "https://api.anthropic.com/v1"

# Path to vmlinux debug symbols (optional, auto-detected)
# vmlinux_path = "/usr/lib/debug/boot/vmlinux-$(uname -r)"

# Enable safety guards
safety_guards = true
require_confirmation = true

# Maximum active probes
max_probes = 16

# Logging level
log_level = "info"
```

Or set your API key as an environment variable:

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

### Installing Kernel Debug Symbols

For full functionality, install kernel debug symbols:

**Ubuntu/Debian:**
```bash
sudo apt-get install linux-image-$(uname -r)-dbgsym
```

**RHEL/CentOS:**
```bash
sudo debuginfo-install kernel
```

**Arch Linux:**
```bash
# Debug symbols are typically in the linux package already
```

## 🚀 Usage

### Interactive Mode (TUI)

Launch the interactive REPL with the beautiful TUI:

```bash
sudo scalpel
```

**TUI Controls:**
- `i` - Enter input mode
- `ESC` - Exit input mode
- `Tab` - Switch focus between panels
- `↑↓` - Scroll logs
- `q` - Quit (when not in input mode)
- `Ctrl+C` - Force quit

**Example Commands:**
```
> trace tcp_v4_rcv where it drops packets
> inspect variable skb->len in tcp_transmit_skb
> hook sys_read and log the filename
> list
> detach <probe-id>
> help
```

### Command Line Mode

Run one-shot analyses:

```bash
# Analyze a request and show the operation plan
sudo scalpel analyze "trace tcp_v4_rcv where it drops packets"

# List active probes
sudo scalpel list

# Detach a specific probe
sudo scalpel detach <probe-id>

# Show configuration
sudo scalpel config --show
```

## 📖 Examples

### Example 1: Trace Function Calls

```bash
> trace tcp_v4_rcv
```

Scalpel will:
1. Resolve `tcp_v4_rcv` in `/proc/kallsyms`
2. Generate a basic kprobe to log every invocation
3. Attach the probe and stream events

### Example 2: Conditional Tracing

```bash
> trace sys_read when fd equals 3
```

Scalpel will:
1. Identify the syscall entry point
2. Generate eBPF code to check the `fd` argument
3. Log only when the condition is met

### Example 3: Inspect Data Structures

```bash
> inspect skb->len in tcp_transmit_skb
```

Scalpel will:
1. Parse the `struct sk_buff` definition
2. Calculate field offsets
3. Generate code to safely read the value

### Example 4: Userspace Hooks

```bash
> hook malloc in libc
```

Scalpel will:
1. Locate `libc.so` via `ldconfig`
2. Calculate the offset of `malloc`
3. Attach a uprobe to trace allocations

### Example 5: Dangerous Mutations (with Confirmation)

```bash
> modify return value of tcp_v4_rcv to 0
```

Scalpel will:
1. Generate BPF code with `bpf_override_return`
2. Display a red "DANGER" warning with risks
3. Request explicit confirmation before proceeding
4. Check if `CONFIG_BPF_KPROBE_OVERRIDE` is enabled

## 🛡️ Safety

Scalpel includes multiple safety layers:

1. **BPF Verifier**: All code must pass kernel verification
2. **Read-Only by Default**: Only explicit mutations trigger warnings
3. **Confirmation Prompts**: Dangerous operations require approval
4. **Capability Checks**: Verifies `CONFIG_BPF_KPROBE_OVERRIDE` before mutations
5. **Risk Assessment**: AI analyzes and reports potential issues

**Mutation Operations Require:**
- Explicit user confirmation
- `CONFIG_BPF_KPROBE_OVERRIDE=y` in kernel
- Root or `CAP_BPF` permissions
- Safety guards enabled in config

## 🏗️ Technology Stack

- **Runtime**: Rust 🦀
- **eBPF Framework**: libbpf (with CO-RE)
- **TUI**: ratatui (Claude Code aesthetic)
- **LLM**: Anthropic Claude 3.5 Sonnet
- **Symbol Resolution**: `/proc/kallsyms`
- **Debug Info**: DWARF parsing (planned: gimli)
- **Disassembly**: Planned integration with capstone

## 🧪 Development

### Building

```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

### Project Structure

```
scalpel/
├── src/
│   ├── main.rs              # Entry point
│   ├── cli.rs               # Command-line interface
│   ├── config.rs            # Configuration management
│   ├── error.rs             # Error types
│   ├── surgical_loop.rs     # Main state machine
│   ├── kernel/              # Kernel analysis
│   │   ├── symbol.rs        # Symbol resolution
│   │   ├── dwarf.rs         # DWARF parsing
│   │   ├── probe.rs         # Probe management
│   │   └── vitals.rs        # Kernel metrics
│   ├── llm/                 # Claude integration
│   │   ├── client.rs        # API client
│   │   └── prompt.rs        # Prompt engineering
│   ├── bpf/                 # eBPF code generation
│   │   ├── codegen.rs       # Code generator
│   │   └── loader.rs        # BPF loader
│   └── tui/                 # Terminal UI
│       ├── app.rs           # Application state
│       ├── ui.rs            # Main layout
│       ├── vitals.rs        # VitalsPanel
│       ├── disassembly.rs   # DisassemblyView
│       ├── probes.rs        # ProbeStatus
│       ├── logs.rs          # LogStream
│       └── input.rs         # Input handling
├── Cargo.toml               # Dependencies
└── README.md                # This file
```

## 🤝 Contributing

Contributions are welcome! Areas for improvement:

- [ ] Full DWARF parsing with gimli
- [ ] Capstone integration for disassembly
- [ ] BTF (BPF Type Format) support
- [ ] Userspace library tracing
- [ ] Historical event playback
- [ ] Export to bpftrace/bcc scripts
- [ ] Multi-architecture support (ARM64)

## ⚠️ Limitations

**Current Version (v1.0.0):**
- eBPF compilation is simulated (requires actual clang integration)
- DWARF parsing returns mock data (needs gimli integration)
- Disassembly is placeholder (needs capstone)
- Semantic location finding uses heuristics (needs AI + disasm)

**System Requirements:**
- Linux kernel 5.8+ with eBPF
- Root or CAP_BPF capability
- Debug symbols for full functionality
- Active internet for Claude API

## 📝 License

MIT License - See LICENSE file for details

## 🙏 Acknowledgments

- Inspired by [bpftrace](https://github.com/iovisor/bpftrace) and [bcc](https://github.com/iovisor/bcc)
- Built with the amazing [aya-rs](https://github.com/aya-rs/aya) ecosystem
- UI inspired by Claude Code's terminal aesthetic
- Powered by Anthropic's Claude 3.5 Sonnet

## 📞 Support

- **Issues**: https://github.com/mnhcorp/scalpel/issues
- **Discussions**: https://github.com/mnhcorp/scalpel/discussions

---

**Made with ❤️ and surgical precision**

*"The kernel is the patient, Scalpel is the surgeon"*
