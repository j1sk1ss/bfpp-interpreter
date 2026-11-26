mod compiler;
mod tape;
mod vm;

pub use compiler::{CompileError, Dialect, compile};
pub use tape::TapeMode;
pub use vm::RuntimeError;

#[derive(Debug)]
pub struct Program {
    instructions: Vec<Instruction>,
    labels: [Option<usize>; 26],
}

impl Program {
    fn label_address(&self, label: u8) -> usize {
        self.labels[label_index(label)].expect("the compiler validates label references")
    }
}

fn label_index(label: u8) -> usize {
    usize::from(label - b'a')
}

#[derive(Clone, Copy, Debug)]
enum Instruction {
    Add(i32),
    Move(i32),
    Read,
    Write,
    JumpIfZero(usize),
    JumpIfNonzero(usize),
    Clear,
    Jump(u8),
    Call(u8),
    ReturnOrHalt,
    LoadArgument(usize),
    StoreArgument(usize),
    LoadReturn,
}
