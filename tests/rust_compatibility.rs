use bfpp::{Dialect, TapeMode, compile};

#[test]
fn adjacent_moves_can_cancel_at_the_left_edge() {
    let program = compile(b"<>+.", Dialect::Bf).unwrap();

    for mode in [TapeMode::Fixed, TapeMode::Grow] {
        let mut output = Vec::new();
        program.run(mode, &[][..], &mut output).unwrap();

        assert_eq!(output, [1]);
    }
}

#[test]
fn invalid_label_operand_reports_the_source_byte_offset() {
    let error = compile("é^A".as_bytes(), Dialect::Bfpp).unwrap_err();

    assert_eq!(
        error.to_string(),
        "compile error [bf_err_syntax] at 2: '^' must be followed by a label a-z"
    );
}

#[test]
fn call_depth_includes_the_root_frame() {
    let program = compile(b"a.&a", Dialect::Bfpp).unwrap();
    let mut output = Vec::new();

    let error = program
        .run(TapeMode::Grow, &[][..], &mut output)
        .unwrap_err();

    assert_eq!(output, [0; 128]);
    assert_eq!(
        error.to_string(),
        "runtime error [bf_err_call_depth] at pc=1: maximum call depth (128) exceeded"
    );
}
