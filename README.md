# Lox Bytecode VM in Rust

My take on the Crafting Interpreters clox bytecode virtual machine using Rust.

Features and highlights:

- Zero LLM-generated code.
- Virtual machine runtime that process compiled bytecode.
- Single-pass compiler: bytecode is emmited on the first pass of the compiler.
- Pratt parser to parse expressions: works in single-pass mode as well - all expressions emit bytecode the moment it's parsed instead of constructing intermediate AST.
