use std::{error::Error, fmt};

use crate::{Instruction, Program, label_index};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dialect {
    Bf,
    Bfpp,
}

#[derive(Debug)]
pub struct CompileError {
    offset: usize,
    message: String,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "compile error [bf_err_syntax] at {}: {}",
            self.offset, self.message
        )
    }
}

impl Error for CompileError {}

pub fn compile(source: &[u8], dialect: Dialect) -> Result<Program, CompileError> {
    let mut instructions = Vec::new();
    let mut loops = Vec::new();
    let mut labels = [None; 26];
    let mut in_comment = false;
    let mut offset = 0;

    while let Some(&byte) = source.get(offset) {
        if dialect == Dialect::Bfpp && byte == b'/' {
            in_comment = !in_comment;
            offset += 1;
            continue;
        }
        if in_comment {
            offset += 1;
            continue;
        }

        let instruction = match byte {
            b'+' | b'-' => {
                offset += emit_run(
                    &mut instructions,
                    &source[offset..],
                    b'+',
                    b'-',
                    Instruction::Add,
                );
                continue;
            }
            b'>' | b'<' => {
                offset += emit_run(
                    &mut instructions,
                    &source[offset..],
                    b'>',
                    b'<',
                    Instruction::Move,
                );
                continue;
            }
            b',' => Some(Instruction::Read),
            b'.' => Some(Instruction::Write),
            b'[' => {
                loops.push(instructions.len());
                Some(Instruction::JumpIfZero(0))
            }
            b']' => {
                let open = loops.pop().ok_or_else(|| CompileError {
                    offset,
                    message: format!("unmatched closing bracket at byte {offset}"),
                })?;
                instructions[open] = Instruction::JumpIfZero(instructions.len() + 1);
                Some(Instruction::JumpIfNonzero(open + 1))
            }
            _ if dialect == Dialect::Bfpp => match byte {
                b'*' => Some(Instruction::Clear),
                b'!' => Some(Instruction::ReturnOrHalt),
                b'$' => Some(Instruction::LoadReturn),
                b'^' | b'&' => {
                    let Some(&label @ b'a'..=b'z') = source.get(offset + 1) else {
                        return Err(CompileError {
                            offset,
                            message: format!(
                                "'{}' must be followed by a label a-z",
                                char::from(byte)
                            ),
                        });
                    };
                    offset += 1;
                    Some(if byte == b'^' {
                        Instruction::Jump(label)
                    } else {
                        Instruction::Call(label)
                    })
                }
                b'#' => {
                    let Some(&digit @ b'0'..=b'9') = source.get(offset + 1) else {
                        return Err(CompileError {
                            offset,
                            message: "'#' must be followed by an argument index 0-9".into(),
                        });
                    };
                    offset += 1;
                    Some(Instruction::LoadArgument(usize::from(digit - b'0')))
                }
                b'0'..=b'9' => Some(Instruction::StoreArgument(usize::from(byte - b'0'))),
                b'a'..=b'z' => {
                    labels[label_index(byte)].get_or_insert(instructions.len());
                    None
                }
                _ => None,
            },
            _ => None,
        };
        instructions.extend(instruction);
        offset += 1;
    }

    if in_comment {
        return Err(CompileError {
            offset: source.len(),
            message: "unterminated '/' comment".into(),
        });
    }
    if !loops.is_empty() {
        return Err(CompileError {
            offset: source.len(),
            message: "unmatched opening bracket".into(),
        });
    }

    for &instruction in &instructions {
        if let Instruction::Jump(label) | Instruction::Call(label) = instruction
            && labels[label_index(label)].is_none()
        {
            return Err(CompileError {
                offset: 0,
                message: format!("undefined label '{}'", char::from(label)),
            });
        }
    }

    Ok(Program {
        instructions,
        labels,
    })
}

fn emit_run(
    instructions: &mut Vec<Instruction>,
    source: &[u8],
    positive: u8,
    negative: u8,
    instruction: fn(i32) -> Instruction,
) -> usize {
    let mut delta = 0_i64;
    let mut length = 0;
    for &byte in source {
        if byte == positive {
            delta += 1;
        } else if byte == negative {
            delta -= 1;
        } else {
            break;
        }
        length += 1;
    }

    while delta != 0 {
        let chunk = delta.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
        instructions.push(instruction(chunk));
        delta -= i64::from(chunk);
    }
    length
}
