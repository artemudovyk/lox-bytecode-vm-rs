use compiler::{Chunk, OpCode, Value};

fn main() {
    // let mut chunk = Chunk::new();
    // chunk.push_code(OpCode::Return);
    // chunk.disassemble("test chunk");

    let mut chunk = Chunk::new();
    let idx = chunk.push_constant(Value::new(1.23));
    chunk.push(OpCode::Constant);
    chunk.push(idx);
    dbg!(&chunk);
    chunk.disassemble("test_chunk");
}
