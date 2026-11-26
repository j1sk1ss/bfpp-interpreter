use std::{env, ffi::OsString, fs, io, process::ExitCode};

use bfpp::{Dialect, TapeMode, compile};

enum CliError {
    Usage,
    Failure(String),
}

fn main() -> ExitCode {
    let mut args = env::args_os();
    let binary = args.next().unwrap_or_else(|| "bfpp".into());

    match run(&args.collect::<Vec<_>>()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::Usage) => {
            eprintln!(
                "Usage: {} [--dialect=bf|bfpp] [--tape=fixed|grow] <file>\n  fixed: 30000 cells\n  grow : starts at 1024 cells, grows up to 1 MiB",
                binary.to_string_lossy()
            );
            ExitCode::from(2)
        }
        Err(CliError::Failure(message)) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[OsString]) -> Result<(), CliError> {
    let mut dialect = b"bfpp".as_slice();
    let mut tape_mode = b"grow".as_slice();
    let mut path = None;

    for arg in args {
        let bytes = arg.as_encoded_bytes();
        if let Some(value) = bytes.strip_prefix(b"--dialect=") {
            dialect = value;
        } else if let Some(value) = bytes.strip_prefix(b"--tape=") {
            tape_mode = value;
        } else if bytes.starts_with(b"-") || path.is_some() {
            return Err(CliError::Usage);
        } else {
            path = Some(arg);
        }
    }

    let path = path.ok_or(CliError::Usage)?;
    let source = fs::read(path).map_err(|error| {
        CliError::Failure(format!("cannot read '{}': {error}", path.to_string_lossy()))
    })?;
    let dialect = match dialect {
        b"bf" => Dialect::Bf,
        b"bfpp" => Dialect::Bfpp,
        value => {
            return Err(CliError::Failure(format!(
                "unknown dialect: {}",
                String::from_utf8_lossy(value)
            )));
        }
    };
    let program =
        compile(&source, dialect).map_err(|error| CliError::Failure(error.to_string()))?;
    let tape_mode = match tape_mode {
        b"fixed" => TapeMode::Fixed,
        b"grow" => TapeMode::Grow,
        value => {
            return Err(CliError::Failure(format!(
                "unknown tape: {}",
                String::from_utf8_lossy(value)
            )));
        }
    };

    program
        .run(tape_mode, io::stdin().lock(), io::stdout().lock())
        .map_err(|error| CliError::Failure(error.to_string()))
}
