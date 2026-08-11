#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    Return,
    LoadConst, // operand: 2 bytes (index to constant pool)
    LoadVar,   // operand: 2 bytes (index to constant pool for variable name)
    StoreVar,  // operand: 2 bytes (index to constant pool for variable name)
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    And,
    Or,
    Not,
    Print,
    JumpIfFalse, // operand: 2 bytes (jump offset)
    Jump,        // operand: 2 bytes (jump offset)
    Call,        // operand: 1 byte (arg count)
    GetIndex,    // operand: 1 byte (index count)
    MakeArray,   // operand: 2 bytes (element count)
    MakeKamus,   // operand: 2 bytes (pair count)
    SetupCatch,  // operand: 2 bytes (jump offset)
    PopCatch,
    Throw,
    Negate,
    LoadModule, // operand: 2 bytes (index to constant pool string for module path)
    Pop,
    IterInit, // No operand. Pops collection, pushes Iterator(idx).
    IterNext, // operand: 2 bytes (jump offset if exhausted). Pops nothing. Modifies Iterator at top of stack. Pushes key, then value, or jumps.
    SetIndex, // operand: 1 byte (index count)
    Power,
    UlangiInit,
    UlangiNext, // operand: 2 bytes (jump offset if exhausted)
    BitwiseOr,
    JumpIfTrue,      // operand: 2 bytes
    JumpIfNotKosong, // operand: 2 bytes
    MakeRentang,     // operand: 1 byte (bitmask: 1 = has_start, 2 = has_end)
}

impl OpCode {
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(OpCode::Return),
            1 => Some(OpCode::LoadConst),
            2 => Some(OpCode::LoadVar),
            3 => Some(OpCode::StoreVar),
            4 => Some(OpCode::Add),
            5 => Some(OpCode::Subtract),
            6 => Some(OpCode::Multiply),
            7 => Some(OpCode::Divide),
            8 => Some(OpCode::Modulus),
            9 => Some(OpCode::Equal),
            10 => Some(OpCode::NotEqual),
            11 => Some(OpCode::Greater),
            12 => Some(OpCode::Less),
            13 => Some(OpCode::GreaterEqual),
            14 => Some(OpCode::LessEqual),
            15 => Some(OpCode::And),
            16 => Some(OpCode::Or),
            17 => Some(OpCode::Not),
            18 => Some(OpCode::Print),
            19 => Some(OpCode::JumpIfFalse),
            20 => Some(OpCode::Jump),
            21 => Some(OpCode::Call),
            22 => Some(OpCode::GetIndex),
            23 => Some(OpCode::MakeArray),
            24 => Some(OpCode::MakeKamus),
            25 => Some(OpCode::SetupCatch),
            26 => Some(OpCode::PopCatch),
            27 => Some(OpCode::Throw),
            28 => Some(OpCode::Negate),
            29 => Some(OpCode::LoadModule),
            30 => Some(OpCode::Pop),
            31 => Some(OpCode::IterInit),
            32 => Some(OpCode::IterNext),
            33 => Some(OpCode::SetIndex),
            34 => Some(OpCode::Power),
            35 => Some(OpCode::UlangiInit),
            36 => Some(OpCode::UlangiNext),
            37 => Some(OpCode::BitwiseOr),
            38 => Some(OpCode::JumpIfTrue),
            39 => Some(OpCode::JumpIfNotKosong),
            40 => Some(OpCode::MakeRentang),
            _ => None,
        }
    }
}
