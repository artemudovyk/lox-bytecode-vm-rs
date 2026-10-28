use compiler::{Chunk, OpCode, Value, Vm};

fn main() {
    // let mut chunk = Chunk::new();
    // chunk.push_code(OpCode::Return);
    // chunk.disassemble("test chunk");

    let mut chunk = Chunk::new();
    let idx = chunk.push_constant(Value::new(1.23));
    chunk.push(OpCode::Constant, 1);
    chunk.push(idx, 1);
    // chunk.disassemble("test_chunk");

    let mut vm = Vm::new();
    vm.enable_tracing();
    vm.interpret(chunk).unwrap();
}
