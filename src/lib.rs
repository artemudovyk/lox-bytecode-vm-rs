use crate::value::Value;
use std::fmt::Display;

pub mod compiler;
pub mod lexer;
pub mod value;
pub mod vm;

/// Operation Code
#[repr(u8)]
#[derive(Debug)]
pub enum OpCode {
    Constant,
    True,
    False,
    Nil,
    Negate,
    Add,
    Sub,
    Mult,
    Div,
    Not,
    Equal,
    Greater,
    Less,
    Return,
    Print,
    /// Used with expression statements to discard the result pushed to the stack.
    Pop,
    DefineGlobal,
}

impl TryFrom<u8> for OpCode {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == OpCode::Return as u8 => Ok(OpCode::Return),
            x if x == OpCode::Constant as u8 => Ok(OpCode::Constant),
            x if x == OpCode::True as u8 => Ok(OpCode::True),
            x if x == OpCode::False as u8 => Ok(OpCode::False),
            x if x == OpCode::Nil as u8 => Ok(OpCode::Nil),
            x if x == OpCode::Add as u8 => Ok(OpCode::Add),
            x if x == OpCode::Sub as u8 => Ok(OpCode::Sub),
            x if x == OpCode::Mult as u8 => Ok(OpCode::Mult),
            x if x == OpCode::Div as u8 => Ok(OpCode::Div),
            x if x == OpCode::Not as u8 => Ok(OpCode::Not),
            x if x == OpCode::Equal as u8 => Ok(OpCode::Equal),
            x if x == OpCode::Greater as u8 => Ok(OpCode::Greater),
            x if x == OpCode::Less as u8 => Ok(OpCode::Less),
            x if x == OpCode::Negate as u8 => Ok(OpCode::Negate),
            x if x == OpCode::Print as u8 => Ok(OpCode::Print),
            x if x == OpCode::Pop as u8 => Ok(OpCode::Pop),
            x if x == OpCode::DefineGlobal as u8 => Ok(OpCode::DefineGlobal),
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
            Self::Constant => write!(f, "OP_CONSTANT"),
            Self::True => write!(f, "OP_TRUE"),
            Self::False => write!(f, "OP_FALSE"),
            Self::Nil => write!(f, "OP_NIL"),
            Self::Negate => write!(f, "OP_NEGATE"),
            Self::Add => write!(f, "OP_ADD"),
            Self::Sub => write!(f, "OP_SUBTRACT"),
            Self::Mult => write!(f, "OP_MULTIPLY"),
            Self::Div => write!(f, "OP_DIVIDE"),
            Self::Not => write!(f, "OP_NOT"),
            Self::Equal => write!(f, "OP_EQUAL"),
            Self::Greater => write!(f, "OP_GREATER"),
            Self::Less => write!(f, "OP_LESS"),
            Self::Return => write!(f, "OP_RETURN"),
            Self::Print => write!(f, "OP_PRINT"),
            Self::Pop => write!(f, "OP_POP"),
            Self::DefineGlobal => write!(f, "OP_DEFINE_GLOBAL"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Chunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    // Run-length encoded code to its line where it occurred. Each index = line. To determine a line
    // of a specific code take the code's index in `self.code[]` and accumulate values starting from
    // the beginning up to the code's index. When the accumulated value exceeds the offset, then the
    // code is on the current line (`i + 1` from `self.lines[]`). It's implemented in the
    // Ref: https://en.wikipedia.org/wiki/Run-length_encoding
    lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            // Init vec with a capacity and fill with zeroes to start counting elements in each line.
            lines: vec![0; 10],
        }
    }

    pub fn push<T: Into<u8>>(&mut self, code: T, line: usize) {
        let code = code.into();
        self.code.push(code);
        self.push_line(line);
    }

    fn push_line(&mut self, line: usize) {
        if self.lines.capacity() < line {
            // `.resize_with()` will increase capacity and initialize newly added values with
            // default value (in our case it will be zeroes).
            self.lines
                .resize_with((self.lines.len() as f64 * 1.5) as usize, Default::default);
        }
        // Lines will always start from 1, but we subtract 1 from it to utilize a value at `[0]`.
        self.lines[line - 1] += 1;
    }

    /// Appends a new constant and returns its index.
    pub fn push_constant(&mut self, val: Value) -> u8 {
        if self.constants.len() > 2 ^ 8 {
            panic!("Too many constants.")
        }
        self.constants.push(val);
        (self.constants.len() - 1) as u8
    }

    fn line_by_code_offset(&self, offset: usize) -> usize {
        // Count all elements that were
        let mut acc = 0;
        for (i, occurrences) in self.lines.iter().enumerate() {
            acc += occurrences;
            if acc > offset {
                return i + 1;
            }
        }
        // TODO: add tests to be sure
        panic!("Line calculation is invalid.")
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
        let line = self.line_by_code_offset(*offset);
        let prev_line = if *offset == 0 {
            0
        } else {
            self.line_by_code_offset(*offset - 1)
        };
        if line == prev_line {
            print!("   | ");
        } else {
            print!("{line:4} ");
        }

        match OpCode::try_from(self.code[*offset]) {
            Ok(op) => match op {
                OpCode::Constant => {
                    let idx = self.code[*offset + 1];
                    let val = &self.constants[idx as usize];
                    println!("{op:<16} [{idx}] {val:>4}");
                    *offset += 2;
                }
                OpCode::DefineGlobal => {
                    let idx = self.code[*offset + 1];
                    let ident = &self.constants[idx as usize];
                    println!("{op:<16} [{idx}] {ident:>4}");
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

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut offset = 0;
        while offset < self.code.len() {
            write!(f, "{offset:04} ")?;
            let line = self.line_by_code_offset(offset);
            let prev_line = if offset == 0 {
                0
            } else {
                self.line_by_code_offset(offset - 1)
            };
            if line == prev_line {
                write!(f, "| ")?;
            } else {
                write!(f, "{line} ")?;
            }

            match OpCode::try_from(self.code[offset]) {
                Ok(op) => match op {
                    OpCode::Constant => {
                        let idx = self.code[offset + 1];
                        let val = &self.constants[idx as usize];
                        writeln!(f, "{op} [{idx}] {val}")?;
                        offset += 2;
                    }
                    OpCode::DefineGlobal => {
                        let idx = self.code[offset + 1];
                        let ident = &self.constants[idx as usize];
                        writeln!(f, "{op} [{idx}] {ident}")?;
                        offset += 2;
                    }
                    _ => {
                        writeln!(f, "{op}")?;
                        offset += 1;
                    }
                },
                Err(_) => panic!("Unknown opcode"),
            }
        }

        Ok(())
    }
}
