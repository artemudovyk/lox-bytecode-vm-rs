use crate::{Chunk, OpCode, value::Value};

pub struct Vm {
    // chunk: Option<Chunk>,
    // src: &'src str,
    cursor: *const u8,
    tracing: bool,
    stack: Vec<Value>,
}

impl Vm {
    const STACK_MAX: usize = 256;

    pub fn new() -> Self {
        Self {
            // chunk: None,
            // src: "",
            cursor: std::ptr::null(),
            tracing: false,
            stack: Vec::with_capacity(Self::STACK_MAX),
        }
    }

    pub fn enable_tracing(&mut self) {
        self.tracing = true;
    }

    pub fn interpret(&mut self, _src: &str) -> Result<(), VmError> {
        // let chunk = self.compiler.compile(self.src).unwrap();
        // self.interpret_chunk(chunk).unwrap();
        Ok(())
    }

    fn interpret_chunk(&mut self, chunk: Chunk) -> Result<(), VmError> {
        // SAFETY:
        // - Each chunk ends with the OpCode::Return.
        // - chunk.code[] is never empty.
        unsafe {
            self.cursor = &chunk.code[0] as *const u8;
            loop {
                if self.tracing {
                    print!("          [");
                    for val in &self.stack {
                        print!(" {} ", val);
                    }
                    println!("]");
                    let start = chunk.code.as_ptr();
                    chunk.disassemble_instruction(
                        // Pointer math to calculate offset from the start of the vec to cursor to reuse
                        // existing `Chunk::disassemble_instruction()` method.
                        &mut ((self.cursor.offset_from(start)) as usize),
                    );
                }
                match OpCode::try_from(*self.cursor)
                    .map_err(|_| RuntimeError::Other("Unknown OpCode.".to_owned()))?
                {
                    OpCode::Constant => {
                        self.move_cursor();
                        self.stack
                            .push(chunk.constants[*self.cursor as usize].clone());
                        self.move_cursor();
                    }
                    OpCode::True => self.stack.push(Value::Bool(true)),
                    OpCode::False => self.stack.push(Value::Bool(false)),
                    OpCode::Nil => self.stack.push(Value::Nil),
                    OpCode::Negate => {
                        let val = match self.stack.pop() {
                            Some(v) => (-v)?,
                            None => {
                                return Err(VmError::RuntimeError(RuntimeError::Other(
                                    "No value on the stack".to_owned(),
                                )));
                            }
                        };
                        self.stack.push(val);
                        self.move_cursor();
                    }
                    OpCode::Add => {
                        // NOTE: we pop values in a backwards order, so we assign the first pop to
                        // v2 instead of v1. Same for all other binary operators.
                        let Some(v2) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        let Some(v1) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        self.stack.push((v1 + v2)?);
                        self.move_cursor();
                    }
                    OpCode::Sub => {
                        let Some(v2) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        let Some(v1) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        self.stack.push((v1 - v2)?);
                        self.move_cursor();
                    }
                    OpCode::Mult => {
                        let Some(v2) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        let Some(v1) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        self.stack.push((v1 * v2)?);
                        self.move_cursor();
                    }
                    OpCode::Div => {
                        let Some(v2) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        let Some(v1) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        self.stack.push((v1 / v2)?);
                        self.move_cursor();
                    }
                    OpCode::Not => {
                        let Some(v) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        self.stack.push(!v);
                        self.move_cursor();
                    }
                    OpCode::Equal => {
                        let Some(v2) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        let Some(v1) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        self.stack.push(Value::Bool(v1 == v2));
                        self.move_cursor();
                    }
                    OpCode::Greater => {
                        let Some(v2) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        let Some(v1) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        self.stack.push(Value::Bool(v1 > v2));
                        self.move_cursor();
                    }
                    OpCode::Less => {
                        let Some(v2) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        let Some(v1) = self.stack.pop() else {
                            return Err(VmError::RuntimeError(RuntimeError::Other(
                                "No value on the stack".to_owned(),
                            )));
                        };
                        self.stack.push(Value::Bool(v1 < v2));
                        self.move_cursor();
                    }
                    OpCode::Return => {
                        match self.stack.pop() {
                            Some(v) => println!("return {v}"),
                            None => println!("return nil"),
                        }
                        return Ok(());
                    }
                    _ => todo!("Unhandled OpCode operator in the interpreter."),
                }
            }
        }
    }

    fn move_cursor(&mut self) {
        unsafe {
            self.cursor = self.cursor.add(1);
        }
    }

    #[allow(dead_code)]
    fn move_cursor_n(&mut self, n: usize) {
        unsafe {
            self.cursor = self.cursor.add(n);
        }
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VmError {
    #[error("compile error")]
    CompileError,
    #[error("runtime error")]
    RuntimeError(#[from] RuntimeError),
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Operand must be a number.")]
    UnsupportedOperands,
    #[error("{0}")]
    Other(String),
}
