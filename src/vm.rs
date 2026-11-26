use std::{
    error::Error,
    fmt,
    io::{self, BufRead, Write},
};

use crate::{Instruction, Program, TapeMode, tape::Tape};

const MAX_CALL_DEPTH: usize = 128;

struct Frame {
    tape: Tape,
    return_address: usize,
    arguments: [u8; 10],
    last_return: u8,
}

impl Frame {
    fn new(tape_mode: TapeMode, return_address: usize) -> Self {
        Self {
            tape: Tape::new(tape_mode),
            return_address,
            arguments: [0; 10],
            last_return: 0,
        }
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pc: usize,
    kind: RuntimeErrorKind,
}

#[derive(Debug)]
enum RuntimeErrorKind {
    TapeBounds,
    CallDepth,
    Output(io::Error),
    Flush(io::Error),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pc = self.pc;
        match self.kind {
            RuntimeErrorKind::TapeBounds => write!(
                f,
                "runtime error [bf_err_tape_bounds] at pc={pc}: tape move out of bounds at pc={pc}"
            ),
            RuntimeErrorKind::CallDepth => write!(
                f,
                "runtime error [bf_err_call_depth] at pc={pc}: maximum call depth ({MAX_CALL_DEPTH}) exceeded"
            ),
            RuntimeErrorKind::Output(_) => {
                write!(
                    f,
                    "runtime error [bf_err_io] at pc={pc}: output error at pc={pc}"
                )
            }
            RuntimeErrorKind::Flush(_) => {
                write!(f, "runtime error [bf_err_io] at pc={pc}: flush failed")
            }
        }
    }
}

impl Error for RuntimeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            RuntimeErrorKind::Output(error) | RuntimeErrorKind::Flush(error) => Some(error),
            RuntimeErrorKind::TapeBounds | RuntimeErrorKind::CallDepth => None,
        }
    }
}

impl Program {
    pub fn run(
        &self,
        tape_mode: TapeMode,
        input: impl BufRead,
        mut output: impl Write,
    ) -> Result<(), RuntimeError> {
        let mut frames = vec![Frame::new(tape_mode, 0)];
        let mut input = input.bytes();
        let mut pc = 0;

        while let Some(&instruction) = self.instructions.get(pc) {
            let (frame, callers) = frames
                .split_last_mut()
                .expect("the root frame stays on the stack");
            let tape = &mut frame.tape;

            match instruction {
                Instruction::Add(delta) => tape.set(tape.get().wrapping_add(delta as u8)),
                Instruction::Move(delta) => {
                    if !tape.move_by(delta) {
                        return Err(RuntimeError {
                            pc,
                            kind: RuntimeErrorKind::TapeBounds,
                        });
                    }
                }
                Instruction::Read => tape.set(input.next().and_then(Result::ok).unwrap_or(0)),
                Instruction::Write => {
                    output
                        .write_all(&[tape.get()])
                        .map_err(|error| RuntimeError {
                            pc,
                            kind: RuntimeErrorKind::Output(error),
                        })?;
                }
                Instruction::JumpIfZero(target) if tape.get() == 0 => {
                    pc = target;
                    continue;
                }
                Instruction::JumpIfNonzero(target) if tape.get() != 0 => {
                    pc = target;
                    continue;
                }
                Instruction::JumpIfZero(_) | Instruction::JumpIfNonzero(_) => {}
                Instruction::Clear => tape.set(0),
                Instruction::Jump(label) => {
                    pc = self.label_address(label);
                    continue;
                }
                Instruction::Call(label) => {
                    if frames.len() == MAX_CALL_DEPTH {
                        return Err(RuntimeError {
                            pc,
                            kind: RuntimeErrorKind::CallDepth,
                        });
                    }
                    frames.push(Frame::new(tape_mode, pc + 1));
                    pc = self.label_address(label);
                    continue;
                }
                Instruction::ReturnOrHalt => {
                    let Some(caller) = callers.last_mut() else {
                        pc = self.instructions.len();
                        break;
                    };
                    caller.last_return = tape.get();
                    pc = frame.return_address;
                    frames.pop();
                    continue;
                }
                Instruction::LoadArgument(index) => {
                    if let Some(caller) = callers.last() {
                        tape.set(caller.arguments[index]);
                    }
                }
                Instruction::StoreArgument(index) => frame.arguments[index] = tape.get(),
                Instruction::LoadReturn => tape.set(frame.last_return),
            }
            pc += 1;
        }

        output.flush().map_err(|error| RuntimeError {
            pc,
            kind: RuntimeErrorKind::Flush(error),
        })
    }
}
