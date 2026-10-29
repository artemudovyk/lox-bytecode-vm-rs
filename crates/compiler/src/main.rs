use compiler::{Chunk, OpCode, Vm, value::Value};

fn main() {
    // let mut chunk = Chunk::new();
    // chunk.push_code(OpCode::Return);
    // chunk.disassemble("test chunk");

    let mut chunk = Chunk::new();
    let idx1 = chunk.push_constant(Value::new(1.23));
    let idx2 = chunk.push_constant(Value::new(2.23));
    let idx3 = chunk.push_constant(Value::new(3.23));
    chunk.push(OpCode::Constant, 1);
    chunk.push(idx1, 1);
    chunk.push(OpCode::Constant, 2);
    chunk.push(idx2, 2);
    chunk.push(OpCode::Constant, 3);
    chunk.push(idx3, 3);
    chunk.push(OpCode::Negate, 3);
    chunk.push(OpCode::Return, 4);
    // chunk.disassemble("test_chunk");

    let mut vm = Vm::new();
    vm.enable_tracing();
    vm.interpret(chunk).unwrap();
}
