use crate::error::{Result, ScalpelError};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub address: u64,
    pub symbol_type: char,
    pub module: Option<String>,
}

impl Symbol {
    pub fn is_function(&self) -> bool {
        matches!(self.symbol_type, 'T' | 't')
    }

    pub fn is_kernel(&self) -> bool {
        self.module.is_none()
    }
}

pub struct SymbolResolver {
    symbols: HashMap<String, Symbol>,
}

impl SymbolResolver {
    pub fn new() -> Result<Self> {
        let mut resolver = Self {
            symbols: HashMap::new(),
        };
        resolver.load_kallsyms()?;
        Ok(resolver)
    }

    fn load_kallsyms(&mut self) -> Result<()> {
        let file = File::open("/proc/kallsyms")
            .map_err(|e| ScalpelError::InsufficientPermissions(
                format!("Cannot read /proc/kallsyms: {}. Try running with sudo.", e)
            ))?;

        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() < 3 {
                continue;
            }

            let address = u64::from_str_radix(parts[0], 16)
                .map_err(|e| ScalpelError::SymbolNotFound(format!("Invalid address: {}", e)))?;
            let symbol_type = parts[1].chars().next().unwrap_or('?');
            let name = parts[2].to_string();
            let module = if parts.len() > 3 {
                Some(parts[3].trim_matches(|c| c == '[' || c == ']').to_string())
            } else {
                None
            };

            self.symbols.insert(name.clone(), Symbol {
                name,
                address,
                symbol_type,
                module,
            });
        }

        tracing::info!("Loaded {} symbols from /proc/kallsyms", self.symbols.len());
        Ok(())
    }

    pub fn resolve(&self, name: &str) -> Result<&Symbol> {
        self.symbols
            .get(name)
            .ok_or_else(|| ScalpelError::SymbolNotFound(name.to_string()))
    }

    pub fn search(&self, pattern: &str) -> Vec<&Symbol> {
        self.symbols
            .values()
            .filter(|s| s.name.contains(pattern))
            .collect()
    }

    pub fn get_function(&self, name: &str) -> Result<&Symbol> {
        let symbol = self.resolve(name)?;
        if !symbol.is_function() {
            return Err(ScalpelError::SymbolNotFound(
                format!("{} is not a function", name)
            ));
        }
        Ok(symbol)
    }
}

impl Default for SymbolResolver {
    fn default() -> Self {
        Self::new().expect("Failed to initialize symbol resolver")
    }
}

impl SymbolResolver {
    pub fn empty() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }
}
