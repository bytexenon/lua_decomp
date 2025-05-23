/*
  Constants for Lua bytecode
*/

use num_enum::TryFromPrimitive;

//////////////////////////////// Variables ////////////////////////////////

// lopcodes.h:211
const TOTAL_OPS: u8 = 38;

//////////////////////////////// Structs ////////////////////////////////

#[derive(Debug)]
pub enum Constant {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Endianness {
    Big,
    Little,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InstructionFormat {
    IABC,
    IABx,
    IAsBx,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OperandMask {
    OpArgN, /* argument is not used */
    OpArgU, /* argument is used */
    OpArgR, /* argument is a register or a jump offset */
    OpArgK, /* argument is a constant or register/constant */
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive)]
#[rustfmt::skip]
#[repr(u8)]
pub enum Opcode {
    MOVE,     LOADK,     LOADBOOL, LOADNIL,
    GETUPVAL, GETGLOBAL, GETTABLE, SETGLOBAL,
    SETUPVAL, SETTABLE,  NEWTABLE, SELF,
    ADD,      SUB,       MUL,      DIV,
    MOD,      POW,       UNM,      NOT,
    LEN,      CONCAT,    JMP,      EQ,
    LT,       LE,        TEST,     TESTSET,
    CALL,     TAILCALL,  RETURN,   FORLOOP,
    FORPREP,  TFORLOOP,  SETLIST,  CLOSE,
    CLOSURE,  VARARG,
}

#[derive(Debug)]
pub struct LocalVariable {
    pub varname: String,
    pub startpc: u32,
    pub endpc: u32,
}

#[derive(Debug)]
pub struct DebugInfo {
    pub lineinfo: Vec<u32>,
    pub locals: Vec<LocalVariable>,
    pub upvalues: Vec<String>,
}

#[derive(Debug)]
pub struct Header {
    pub version: u8,            // Lua version (0x51 for Lua 5.1)
    pub format: u8,             // Bytecode format (0 for official Lua bytecode)
    pub endianness: Endianness, // Byte order (Big or Little Endian)
    pub size_int: u8,           // Size of an integer in bytes
    pub size_size_t: u8,        // Size of a size_t value in bytes
    pub size_instruction: u8,   // Size of an instruction in bytes
    pub size_number: u8,        // Size of a number in bytes
    pub integral_flag: bool,    // Whether numbers are stored as integers or floats
}

#[derive(Debug)]
pub struct FunctionPrototype {
    pub source_name: String,
    pub line_defined: i32,
    pub last_line_defined: i32,
    pub num_upvalues: u8,
    pub num_params: u8,
    pub is_vararg: u8,
    pub max_stack_size: u8,
    pub code: Vec<Instruction>,
    pub constants: Vec<Constant>,
    pub prototypes: Vec<FunctionPrototype>,
    pub debug_info: DebugInfo,
}

#[derive(Debug, Clone)]
pub struct Instruction(u32);
impl Instruction {
    pub const SIZE_OP: u32 = 6;
    pub const SIZE_C: u32 = 9;
    pub const SIZE_B: u32 = 9;
    pub const SIZE_A: u32 = 8;
    pub const SIZE_BX: u32 = Instruction::SIZE_C + Instruction::SIZE_B;
    // const SIZE_SBX: u32 = Instruction::SIZE_BX - 1;

    pub const POS_OP: u32 = 0;
    pub const POS_A: u32 = Instruction::POS_OP + Instruction::SIZE_OP;
    pub const POS_C: u32 = Instruction::POS_A + Instruction::SIZE_A;
    pub const POS_B: u32 = Instruction::POS_C + Instruction::SIZE_C;
    pub const POS_BX: u32 = Instruction::POS_C;
    // const POS_SBX: u32 = Instruction::POS_BX;

    pub const fn new(instr: u32) -> Self {
        Self(instr)
    }

    // Utility Functions //
    const fn extract_bits(start: u32, end: u32, value: u32) -> u32 {
        assert!(start < end && end <= 32, "Invalid bit range");

        let mask = (1 << (end - start)) - 1;
        (value >> start) & mask
    }
    const fn convert_mode(mode: OperandMask) -> &'static str {
        match mode {
            OperandMask::OpArgN => "OpArgN",
            OperandMask::OpArgU => "OpArgU",
            OperandMask::OpArgR => "OpArgR",
            OperandMask::OpArgK => "OpArgK",
        }
    }

    // Instruction Info //
    pub fn opcode(&self) -> Opcode {
        let op = Self::extract_bits(
            Instruction::POS_OP,
            Instruction::POS_OP + Instruction::SIZE_OP,
            self.0,
        ) as u8;
        Opcode::try_from(op).unwrap()
    }

    pub fn format(&self) -> InstructionFormat {
        match OPMODES[self.opcode() as usize].0 {
            InstructionFormat::IABC => InstructionFormat::IABC,
            InstructionFormat::IABx => InstructionFormat::IABx,
            InstructionFormat::IAsBx => InstructionFormat::IAsBx,
        }
    }

    // Operands //

    /* A */
    pub const fn a(&self) -> u32 {
        Self::extract_bits(
            Instruction::POS_A,
            Instruction::POS_A + Instruction::SIZE_A,
            self.0,
        ) as u32
    }

    /* B */
    pub const fn b(&self) -> u32 {
        Self::extract_bits(
            Instruction::POS_C,
            Instruction::POS_C + Instruction::SIZE_C,
            self.0,
        ) as u32
    }
    pub const fn b_isk(&self) -> bool {
        (Self::b(self) & (1 << (9 - 1))) != 0
    }
    pub const fn bk(&self) -> u32 {
        Self::b(self) & !(1 << (9 - 1))
    }
    pub fn b_mode(&self) -> OperandMask {
        OPMODES[self.opcode() as usize].1
    }

    /* C */
    pub const fn c(&self) -> u32 {
        Self::extract_bits(
            Instruction::POS_B,
            Instruction::POS_B + Instruction::SIZE_B,
            self.0,
        ) as u32
    }
    pub const fn c_isk(&self) -> bool {
        (Self::c(self) & (1 << (9 - 1))) != 0
    }
    pub const fn ck(&self) -> u32 {
        Self::c(self) & !(1 << (9 - 1))
    }

    pub fn c_mode(&self) -> OperandMask {
        OPMODES[self.opcode() as usize].2
    }

    /* Special */
    pub const fn bx(&self) -> u32 {
        Self::extract_bits(
            Instruction::POS_BX,
            Instruction::POS_BX + Instruction::SIZE_BX,
            self.0,
        ) as u32
    }

    pub const fn sbx(&self) -> i32 {
        let bx = self.bx() as i32;
        bx - (1 << 17) + 1
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let opcode = OPNAMES[self.opcode() as usize];
        let format = self.format();
        let a = self.a();
        let b = self.b();
        let c = self.c();
        let bx = self.bx();
        let sbx = self.sbx();
        let b_isk = self.b_isk();
        let c_isk = self.c_isk();
        let bk = self.bk();
        let ck = self.ck();
        let b_mode = self.b_mode();
        let c_mode = self.c_mode();
        let b_mode_str = Instruction::convert_mode(b_mode);
        let c_mode_str = Instruction::convert_mode(c_mode);

        write!(f, "Instruction(")?;
        write!(f, "opname: {opcode}")?;
        write!(f, " format: {:?},", format)?;

        // TODO: Opcode-specific printers
        match format {
            InstructionFormat::IABC => {
                if b_mode == OperandMask::OpArgK && b_isk {
                    write!(f, " b: K({}),", bk)?;
                } else {
                    write!(f, " b: {},", b)?;
                }
                if c_mode == OperandMask::OpArgK && c_isk {
                    write!(f, " c: K({}),", ck)?;
                } else {
                    write!(f, " c: {},", c)?;
                }
                write!(f, " a: {},", a)?;
                write!(f, " b_mode: {},", b_mode_str)?;
                write!(f, " c_mode: {},", c_mode_str)?;
            }
            InstructionFormat::IABx => {
                if b_mode == OperandMask::OpArgK {
                    write!(f, " bx: K({}),", bx)?;
                } else {
                    write!(f, " bx: {},", bx)?;
                }
                write!(f, " b_isk: {},", b_isk)?;
            }
            InstructionFormat::IAsBx => {
                write!(f, " sbx: {},", sbx)?;
            }
        }

        write!(f, " raw: {:08x}", self.0)?;
        write!(f, ")")
    }
}

//////////////////////////////// Lookup Tables ////////////////////////////////

#[rustfmt::skip]
const OPMODES: [(InstructionFormat, OperandMask, OperandMask); TOTAL_OPS as usize] = [
    /*    Opcode Format            Operand B            Operand C         */
    (InstructionFormat::IABC, OperandMask::OpArgR, OperandMask::OpArgN),  // OP_MOVE
    (InstructionFormat::IABx, OperandMask::OpArgK, OperandMask::OpArgN),  // OP_LOADK
    (InstructionFormat::IABC, OperandMask::OpArgU, OperandMask::OpArgU),  // OP_LOADBOOL
    (InstructionFormat::IABC, OperandMask::OpArgR, OperandMask::OpArgN),  // OP_LOADNIL
    (InstructionFormat::IABC, OperandMask::OpArgU, OperandMask::OpArgN),  // OP_GETUPVAL
    (InstructionFormat::IABx, OperandMask::OpArgK, OperandMask::OpArgN),  // OP_GETGLOBAL
    (InstructionFormat::IABC, OperandMask::OpArgR, OperandMask::OpArgK),  // OP_GETTABLE
    (InstructionFormat::IABx, OperandMask::OpArgK, OperandMask::OpArgN),  // OP_SETGLOBAL
    (InstructionFormat::IABC, OperandMask::OpArgU, OperandMask::OpArgN),  // OP_SETUPVAL
    (InstructionFormat::IABC, OperandMask::OpArgK, OperandMask::OpArgK),  // OP_SETTABLE
    (InstructionFormat::IABC, OperandMask::OpArgU, OperandMask::OpArgU),  // OP_NEWTABLE
    (InstructionFormat::IABC, OperandMask::OpArgR, OperandMask::OpArgK),  // OP_SELF
    (InstructionFormat::IABC, OperandMask::OpArgK, OperandMask::OpArgK),  // OP_ADD
    (InstructionFormat::IABC, OperandMask::OpArgK, OperandMask::OpArgK),  // OP_SUB
    (InstructionFormat::IABC, OperandMask::OpArgK, OperandMask::OpArgK),  // OP_MUL
    (InstructionFormat::IABC, OperandMask::OpArgK, OperandMask::OpArgK),  // OP_DIV
    (InstructionFormat::IABC, OperandMask::OpArgK, OperandMask::OpArgK),  // OP_MOD
    (InstructionFormat::IABC, OperandMask::OpArgK, OperandMask::OpArgK),  // OP_POW
    (InstructionFormat::IABC, OperandMask::OpArgR, OperandMask::OpArgN),  // OP_UNM
    (InstructionFormat::IABC, OperandMask::OpArgR, OperandMask::OpArgN),  // OP_NOT
    (InstructionFormat::IABC, OperandMask::OpArgR, OperandMask::OpArgN),  // OP_LEN
    (InstructionFormat::IABC, OperandMask::OpArgR, OperandMask::OpArgR),  // OP_CONCAT
    (InstructionFormat::IAsBx, OperandMask::OpArgR, OperandMask::OpArgN), // OP_JMP
    (InstructionFormat::IABC, OperandMask::OpArgK, OperandMask::OpArgK),  // OP_EQ
    (InstructionFormat::IABC, OperandMask::OpArgK, OperandMask::OpArgK),  // OP_LT
    (InstructionFormat::IABC, OperandMask::OpArgK, OperandMask::OpArgK),  // OP_LE
    (InstructionFormat::IABC, OperandMask::OpArgR, OperandMask::OpArgU),  // OP_TEST
    (InstructionFormat::IABC, OperandMask::OpArgR, OperandMask::OpArgU),  // OP_TESTSET
    (InstructionFormat::IABC, OperandMask::OpArgU, OperandMask::OpArgU),  // OP_CALL
    (InstructionFormat::IABC, OperandMask::OpArgU, OperandMask::OpArgU),  // OP_TAILCALL
    (InstructionFormat::IABC, OperandMask::OpArgU, OperandMask::OpArgN),  // OP_RETURN
    (InstructionFormat::IAsBx, OperandMask::OpArgR, OperandMask::OpArgN), // OP_FORLOOP
    (InstructionFormat::IAsBx, OperandMask::OpArgR, OperandMask::OpArgN), // OP_FORPREP
    (InstructionFormat::IABC, OperandMask::OpArgN, OperandMask::OpArgU),  // OP_TFORLOOP
    (InstructionFormat::IABC, OperandMask::OpArgU, OperandMask::OpArgU),  // OP_SETLIST
    (InstructionFormat::IABC, OperandMask::OpArgN, OperandMask::OpArgN),  // OP_CLOSE
    (InstructionFormat::IABx, OperandMask::OpArgU, OperandMask::OpArgN),  // OP_CLOSURE
    (InstructionFormat::IABC, OperandMask::OpArgU, OperandMask::OpArgN),  // OP_VARARG
];

#[rustfmt::skip]
const OPNAMES: [&str; TOTAL_OPS as usize] = [
    "MOVE",     "LOADK",     "LOADBOOL", "LOADNIL",
    "GETUPVAL", "GETGLOBAL", "GETTABLE", "SETGLOBAL",
    "SETUPVAL", "SETTABLE",  "NEWTABLE", "SELF",
    "ADD",      "SUB",       "MUL",      "DIV",
    "MOD",      "POW",       "UNM",      "NOT",
    "LEN",      "CONCAT",    "JMP",      "EQ",
    "LT",       "LE",        "TEST",     "TESTSET",
    "CALL",     "TAILCALL",  "RETURN",   "FORLOOP",
    "FORPREP",  "TFORLOOP",  "SETLIST",  "CLOSE",
    "CLOSURE",  "VARARG",
];
