/// Represents a local variable debug information
#[derive(Debug)]
#[allow(dead_code)]
pub struct LocalVariable {
    pub varname: String, // Variable name
    pub startpc: u32,    // First instruction index where the variable is valid
    pub endpc: u32,      // Last instruction index where the variable is valid
}

/// Represents debug information for a function
#[derive(Debug)]
#[allow(dead_code)]
pub struct DebugInfo {
    pub lineinfo: Vec<u32>,         // Line numbers for each instruction
    pub locals: Vec<LocalVariable>, // Local variable information
    pub upvalues: Vec<String>,      // Upvalue names
}
