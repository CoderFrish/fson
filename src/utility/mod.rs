pub fn is_literal(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ['+', '-', '.'].contains(&ch)
}

pub fn is_control(ch: char) -> bool {
    ['\t', '\n', '\r'].contains(&ch)
}
