use std::fmt::Display;

/// Operation Code
#[repr(u8)]
#[derive(Debug)]
pub enum OpCode {
    Return,
    Constant,
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == OpCode::Return as u8 => Ok(OpCode::Return),
            x if x == OpCode::Constant as u8 => Ok(OpCode::Constant),
            _ => Err(()),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(val: OpCode) -> Self {
        val as u8
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Return => write!(f, "OP_RETURN"),
            Self::Constant => write!(f, "OP_CONSTANT"),
        }
    }
}

impl OpCode {}

#[derive(Debug)]
pub struct Value(f64);

impl Value {
    pub fn new(val: f64) -> Self {
        Self(val)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            constants: vec![],
        }
    }

    pub fn push<T: Into<u8>>(&mut self, code: T) {
        let code = code.into();
        self.code.push(code);
    }

    pub fn push_code(&mut self, code: OpCode) {
        self.code.push(code as u8);
    }

    /// Appends a new constant and returns its index.
    pub fn push_constant(&mut self, val: Value) -> u8 {
        if self.constants.len() > 2 ^ 8 {
            panic!("Too many constants.")
        }
        self.constants.push(val);
        (self.constants.len() - 1) as u8
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {name} ==");

        let mut offset = 0;
        while offset < self.code.len() {
            self.disassemble_instruction(&mut offset);
        }
    }

    fn disassemble_instruction(&self, offset: &mut usize) {
        print!("{offset:04} ");

        match OpCode::try_from(self.code[*offset]) {
            Ok(op) => match op {
                OpCode::Constant => {
                    let idx = self.code[*offset + 1];
                    let val = &self.constants[idx as usize];
                    println!("{op:<16} [{idx}] {val:>4}");
                    *offset += 2;
                }
                _ => {
                    println!("{op}");
                    *offset += 1;
                }
            },
            Err(_) => panic!("Unknown opcode"),
        }
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
