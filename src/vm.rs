use std::io::Write;

use crate::{
    Chunk, OpCode,
    compiler::{CompileError, compile},
    value::Value,
};

pub struct Vm<W: Write = std::io::Stdout, E: Write = std::io::Stderr> {
    // chunk: Option<Chunk>,
    // src: &'src str,
    cursor: *const u8,
    tracing: bool,
    stack: Vec<Value>,
    writer: W,
    error_writer: E,
}

impl Vm<std::io::Stdout> {
    pub fn new() -> Self {
        Self::with_writers(std::io::stdout(), std::io::stderr())
    }
}

impl<W: Write, E: Write> Vm<W, E> {
    const STACK_MAX: usize = 256;

    pub fn with_writers(writer: W, error_writer: E) -> Self {
        Self {
            // chunk: None,
            // src: "",
            cursor: std::ptr::null(),
            tracing: false,
            stack: Vec::with_capacity(Self::STACK_MAX),
            writer,
            error_writer,
        }
    }

    pub fn enable_tracing(&mut self) {
        self.tracing = true;
    }

    pub fn interpret(&mut self, src: &str) -> Result<(), VmError> {
        let chunk = match compile(src) {
            Ok(v) => v,
            Err(errors) => {
                for error in &errors {
                    writeln!(self.error_writer, "{}", error)?;
                }
                return Err(VmError::CompileErrors(errors));
            }
        };
        chunk.disassemble("chunk");
        self.interpret_chunk(chunk)?;
        Ok(())
    }

    fn interpret_chunk(&mut self, chunk: Chunk) -> Result<(), VmError> {
        if chunk.code.is_empty() {
            todo!("Return error that the chunk bytecode vec is empty");
        }

        chunk.disassemble("test");

        unsafe {
            self.cursor = &chunk.code[0] as *const u8;
            let last = &chunk.code[chunk.code.len() - 1] as *const u8;

            while last.offset_from(self.cursor) >= 0 {
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
                    OpCode::Print => match self.stack.pop() {
                        Some(v) => {
                            writeln!(self.writer, "{}", v)?;
                            self.move_cursor()
                        }
                        None => writeln!(self.error_writer, "Nothing on the stack.")?,
                    },
                    OpCode::Pop => {
                        self.stack.pop();
                    }
                }
            }

            Ok(())
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

impl Default for Vm<std::io::Stdout> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum VmError {
    #[error("compile errors")]
    CompileErrors(Vec<CompileError>),
    #[error("runtime error")]
    RuntimeError(#[from] RuntimeError),
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("Operand must be a number.")]
    UnsupportedOperands,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    Other(String),
}

// This allows to bubble up the io::Error without mapping where VmError is expected (e.g. `Vm::interpret()`).
impl From<std::io::Error> for VmError {
    fn from(value: std::io::Error) -> Self {
        Self::RuntimeError(RuntimeError::from(value))
    }
}

impl From<Vec<CompileError>> for VmError {
    fn from(err: Vec<CompileError>) -> Self {
        VmError::CompileErrors(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_stmt() {
        let src = r#"print "my-string";"#;
        let mut buf: Vec<u8> = Vec::new();
        let mut vm = Vm::with_writers(&mut buf, std::io::stderr());
        vm.interpret(src).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "\"my-string\"\n");
    }

    #[test]
    fn print_expression_output() {
        let src = r#"print 1 + 2 * 3;"#;
        let mut buf: Vec<u8> = Vec::new();
        let mut vm = Vm::with_writers(&mut buf, std::io::stderr());
        vm.interpret(src).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "7.0\n");
    }

    #[test]
    fn multiple_statements() {
        let src = r#"
print 1 + 2 * 3;
print 2 + 3 * 4;
"#;
        let mut buf: Vec<u8> = Vec::new();
        let mut vm = Vm::with_writers(&mut buf, std::io::stderr());
        vm.tracing = true;
        vm.interpret(src).unwrap();
        assert_eq!(std::str::from_utf8(&buf).unwrap(), "7.0\n14.0\n");
    }

    #[test]
    fn capture_error() {
        let src = r#"1 + 1"#;
        let mut buf: Vec<u8> = Vec::new();
        let mut errbuf: Vec<u8> = Vec::new();
        let mut vm = Vm::with_writers(&mut buf, &mut errbuf);
        vm.tracing = true;
        let _ = vm.interpret(src);
        assert_eq!(
            std::str::from_utf8(&errbuf).unwrap(),
            "[line 1] Error at '1': Expect ';' at the end of the expression statement.\n"
        );
    }

    #[test]
    fn capture_multiple_errors() {
        let src = r#"
1 + (2 * 3;
2 + 4"#;
        let mut buf: Vec<u8> = Vec::new();
        let mut errbuf: Vec<u8> = Vec::new();
        let mut vm = Vm::with_writers(&mut buf, &mut errbuf);
        vm.tracing = true;
        let _ = vm.interpret(src);
        assert_eq!(
            std::str::from_utf8(&errbuf).unwrap(),
            "[line 2] Error at ';': Expect ')' at the end of the grouping.\n[line 3] Error at '4': Expect ';' at the end of the expression statement.\n"
        );
    }
}
