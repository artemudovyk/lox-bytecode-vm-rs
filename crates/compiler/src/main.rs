use compiler::{Chunk, OpCode, Vm, value::Value};

fn main() {
    // let mut chunk = Chunk::new();
    // chunk.push_code(OpCode::Return);
    // chunk.disassemble("test chunk");

    let mut chunk = Chunk::new();
    let idx1 = chunk.push_constant(Value::new(1.2));
    let idx2 = chunk.push_constant(Value::new(3.4));
    let idx3 = chunk.push_constant(Value::new(5.6));
    chunk.push(OpCode::Constant, 1);
    chunk.push(idx1, 1);
    chunk.push(OpCode::Constant, 1);
    chunk.push(idx2, 1);
    chunk.push(OpCode::Add, 1);
    chunk.push(OpCode::Constant, 1);
    chunk.push(idx3, 1);
    chunk.push(OpCode::Div, 1);
    chunk.push(OpCode::Negate, 1);
    chunk.push(OpCode::Return, 2);
    // chunk.disassemble("test_chunk");

    let mut vm = Vm::new();
    vm.enable_tracing();
    vm.interpret(chunk).unwrap();
}
