use compiler::{Chunk, OpCode, Value};

fn main() {
    // let mut chunk = Chunk::new();
    // chunk.push_code(OpCode::Return);
    // chunk.disassemble("test chunk");

    let mut chunk = Chunk::new();
    let idx = chunk.push_constant(Value::new(1.23));
    chunk.push(OpCode::Constant, 1);
    chunk.push(idx, 1);
    chunk.push(OpCode::Constant, 1);
    chunk.push(idx, 1);
    chunk.push(OpCode::Constant, 2);
    chunk.push(idx, 2);
    chunk.push(OpCode::Constant, 2);
    chunk.push(idx, 2);
    chunk.push(OpCode::Constant, 3);
    chunk.push(idx, 3);
    chunk.disassemble("test_chunk");
}
