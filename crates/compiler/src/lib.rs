use std::{collections::HashMap, fmt::Display};

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
        // Lines will always start from 1, but we substract 1 from it to utilize a value at `[0]`.
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
