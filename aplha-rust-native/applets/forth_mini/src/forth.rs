pub const INPUT_CAPACITY: usize = 32;
pub const LINE_WIDTH: usize = 28;
pub const OUTPUT_LINES: usize = 3;

const STACK_CAPACITY: usize = 16;

#[derive(Clone, Copy, Eq, PartialEq)]
enum EvalError {
    DivideByZero,
    InputFull,
    StackOverflow,
    StackUnderflow,
    UnknownWord,
}

#[repr(C)]
#[derive(Clone, Eq, PartialEq)]
pub struct Repl {
    stack: [i32; STACK_CAPACITY],
    depth: usize,
    input: [u8; INPUT_CAPACITY],
    input_len: usize,
    output_line: [u8; LINE_WIDTH],
    pub output: [[u8; LINE_WIDTH]; OUTPUT_LINES],
}

impl Repl {
    pub const fn new() -> Self {
        Self {
            stack: [0; STACK_CAPACITY],
            depth: 0,
            input: [0; INPUT_CAPACITY],
            input_len: 0,
            output_line: blank_line(),
            output: [blank_line(), blank_line(), blank_line()],
        }
    }

    pub fn accept_printable(&mut self, byte: u8) {
        if !(b' '..=b'~').contains(&byte) {
            return;
        }
        if self.input_len >= INPUT_CAPACITY {
            self.set_error(EvalError::InputFull);
            return;
        }
        self.input[self.input_len] = byte;
        self.input_len += 1;
    }

    pub fn backspace(&mut self) {
        if self.input_len > 0 {
            self.input_len -= 1;
        }
    }

    pub fn enter(&mut self) {
        if self.input_len == 0 {
            return;
        }

        let mut cursor = 0usize;
        while let Some(token) = next_token_copy(&self.input, self.input_len, &mut cursor) {
            if let Err(error) = self.eval_token(&token.bytes[..token.len]) {
                self.set_error(error);
                self.input_len = 0;
                return;
            }
        }
        self.input_len = 0;
    }

    pub fn line_title(&self) -> [u8; LINE_WIDTH] {
        let mut line = blank_line();
        write_bytes(&mut line, 0, b"Forth Mini");
        line
    }

    pub fn line_stack(&self) -> [u8; LINE_WIDTH] {
        let mut line = blank_line();
        write_bytes(&mut line, 0, b"S:<");
        let mut offset = 3usize;
        offset = write_usize(&mut line, offset, self.depth);
        if offset < LINE_WIDTH {
            line[offset] = b'>';
            offset += 1;
        }
        if offset < LINE_WIDTH {
            line[offset] = b' ';
            offset += 1;
        }

        let start = self.depth.saturating_sub(3);
        let mut index = start;
        while index < self.depth && offset < LINE_WIDTH {
            offset = write_i32(&mut line, offset, self.stack[index]);
            if offset < LINE_WIDTH && index + 1 < self.depth {
                line[offset] = b' ';
                offset += 1;
            }
            index += 1;
        }
        line
    }

    pub fn line_output(&self) -> [u8; LINE_WIDTH] {
        self.output_line
    }

    pub fn line_prompt(&self) -> [u8; LINE_WIDTH] {
        let mut line = blank_line();
        line[0] = b'>';
        line[1] = b' ';
        let visible = self.input_len.min(LINE_WIDTH - 2);
        let start = self.input_len.saturating_sub(visible);
        let mut index = 0usize;
        while index < visible {
            line[index + 2] = self.input[start + index];
            index += 1;
        }
        line
    }

    pub fn push_byte(&mut self, byte: u8) {
        self.accept_printable(byte);
    }

    pub fn eval_line(&mut self) {
        self.enter();
    }

    pub fn stack_line(&self) -> [u8; LINE_WIDTH] {
        self.line_stack()
    }

    pub fn prompt_line(&self) -> [u8; LINE_WIDTH] {
        self.line_prompt()
    }

    fn eval_token(&mut self, token: &[u8]) -> Result<(), EvalError> {
        if let Some(value) = parse_i32(token) {
            return self.push(value);
        }

        match token {
            b"+" => self.binary_op(i32::wrapping_add),
            b"-" => self.binary_op(i32::wrapping_sub),
            b"*" => self.binary_op(i32::wrapping_mul),
            b"/" => self.checked_div(),
            b"mod" => self.checked_mod(),
            b"dup" => self.dup(),
            b"drop" => self.drop_top(),
            b"swap" => self.swap(),
            b"over" => self.over(),
            b"." => self.print_top(),
            b".s" => {
                self.set_output(self.line_stack());
                Ok(())
            }
            b"clear" => {
                self.depth = 0;
                self.set_output(line_from_bytes(b"ok"));
                Ok(())
            }
            _ => Err(EvalError::UnknownWord),
        }
    }

    fn push(&mut self, value: i32) -> Result<(), EvalError> {
        if self.depth >= STACK_CAPACITY {
            return Err(EvalError::StackOverflow);
        }
        self.stack[self.depth] = value;
        self.depth += 1;
        Ok(())
    }

    fn pop(&mut self) -> Result<i32, EvalError> {
        if self.depth == 0 {
            return Err(EvalError::StackUnderflow);
        }
        self.depth -= 1;
        Ok(self.stack[self.depth])
    }

    fn binary_op(&mut self, op: fn(i32, i32) -> i32) -> Result<(), EvalError> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        self.push(op(lhs, rhs))
    }

    fn checked_div(&mut self) -> Result<(), EvalError> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        if rhs == 0 {
            return Err(EvalError::DivideByZero);
        }
        self.push(lhs / rhs)
    }

    fn checked_mod(&mut self) -> Result<(), EvalError> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        if rhs == 0 {
            return Err(EvalError::DivideByZero);
        }
        self.push(lhs % rhs)
    }

    fn dup(&mut self) -> Result<(), EvalError> {
        if self.depth == 0 {
            return Err(EvalError::StackUnderflow);
        }
        self.push(self.stack[self.depth - 1])
    }

    fn drop_top(&mut self) -> Result<(), EvalError> {
        self.pop().map(|_| ())
    }

    fn swap(&mut self) -> Result<(), EvalError> {
        if self.depth < 2 {
            return Err(EvalError::StackUnderflow);
        }
        self.stack.swap(self.depth - 1, self.depth - 2);
        Ok(())
    }

    fn over(&mut self) -> Result<(), EvalError> {
        if self.depth < 2 {
            return Err(EvalError::StackUnderflow);
        }
        self.push(self.stack[self.depth - 2])
    }

    fn print_top(&mut self) -> Result<(), EvalError> {
        let value = self.pop()?;
        let mut line = blank_line();
        write_i32(&mut line, 0, value);
        self.set_output(line);
        Ok(())
    }

    fn set_error(&mut self, error: EvalError) {
        let line = match error {
            EvalError::DivideByZero => line_from_bytes(b"divide by zero"),
            EvalError::InputFull => line_from_bytes(b"input full"),
            EvalError::StackOverflow => line_from_bytes(b"stack overflow"),
            EvalError::StackUnderflow => line_from_bytes(b"stack underflow"),
            EvalError::UnknownWord => line_from_bytes(b"unknown word"),
        };
        self.set_output(line);
    }

    fn set_output(&mut self, line: [u8; LINE_WIDTH]) {
        self.output_line = line;
        self.output[0] = blank_line();
        self.output[1] = blank_line();
        self.output[2] = line;
    }
}

const fn blank_line() -> [u8; LINE_WIDTH] {
    [b' '; LINE_WIDTH]
}

fn line_from_bytes(bytes: &[u8]) -> [u8; LINE_WIDTH] {
    let mut line = blank_line();
    write_bytes(&mut line, 0, bytes);
    line
}

fn write_bytes(line: &mut [u8; LINE_WIDTH], offset: usize, bytes: &[u8]) {
    let mut index = 0usize;
    while offset + index < LINE_WIDTH && index < bytes.len() {
        line[offset + index] = bytes[index];
        index += 1;
    }
}

struct Token {
    bytes: [u8; INPUT_CAPACITY],
    len: usize,
}

fn next_token_copy(input: &[u8; INPUT_CAPACITY], input_len: usize, cursor: &mut usize) -> Option<Token> {
    while *cursor < input_len && input[*cursor] == b' ' {
        *cursor += 1;
    }
    let start = *cursor;
    while *cursor < input_len && input[*cursor] != b' ' {
        *cursor += 1;
    }
    if start == *cursor {
        return None;
    }
    let mut token = Token {
        bytes: [0u8; INPUT_CAPACITY],
        len: 0,
    };
    let mut index = 0usize;
    while start + index < *cursor && index < INPUT_CAPACITY {
        token.bytes[index] = input[start + index];
        index += 1;
    }
    token.len = index;
    Some(token)
}

fn parse_i32(token: &[u8]) -> Option<i32> {
    if token.is_empty() {
        return None;
    }

    let negative = token[0] == b'-';
    let start = usize::from(negative);
    if start == token.len() {
        return None;
    }

    let mut value = 0i32;
    let mut index = start;
    while index < token.len() {
        let byte = token[index];
        if !byte.is_ascii_digit() {
            return None;
        }
        value = value.checked_mul(10)?;
        let digit = i32::from(byte - b'0');
        value = if negative {
            value.checked_sub(digit)?
        } else {
            value.checked_add(digit)?
        };
        index += 1;
    }
    Some(value)
}

fn write_usize(line: &mut [u8; LINE_WIDTH], offset: usize, value: usize) -> usize {
    let Ok(value) = i32::try_from(value) else {
        return offset;
    };
    write_i32(line, offset, value)
}

fn write_i32(line: &mut [u8; LINE_WIDTH], mut offset: usize, value: i32) -> usize {
    if offset >= LINE_WIDTH {
        return offset;
    }

    if value == 0 {
        line[offset] = b'0';
        return offset + 1;
    }

    let mut digits = [0u8; 11];
    let mut count = 0usize;
    let mut work = value.unsigned_abs();
    while work > 0 && count < digits.len() {
        digits[count] = b'0' + (work % 10) as u8;
        work /= 10;
        count += 1;
    }

    if value < 0 {
        line[offset] = b'-';
        offset += 1;
    }

    while count > 0 && offset < LINE_WIDTH {
        count -= 1;
        line[offset] = digits[count];
        offset += 1;
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_printable_without_evaluating_until_enter() {
        let mut repl = Repl::new();

        repl.accept_printable(b'1');

        assert_eq!(repl.line_output(), blank_line());
        assert_eq!(repl.line_prompt()[..3], [b'>', b' ', b'1']);
    }

    #[test]
    fn enter_evaluates_current_input_line() {
        let mut repl = Repl::new();

        repl.accept_printable(b'1');
        repl.enter();

        assert_eq!(repl.line_stack()[..5], [b'S', b':', b'<', b'1', b'>']);
    }

    #[test]
    fn backspace_removes_one_input_byte() {
        let mut repl = Repl::new();

        repl.accept_printable(b'1');
        repl.accept_printable(b'2');
        repl.backspace();

        assert_eq!(repl.line_prompt()[..3], [b'>', b' ', b'1']);
    }

    #[test]
    fn evaluates_arithmetic_and_prints() {
        let mut repl = Repl::new();
        repl.accept_printable(b'2');
        repl.enter();
        repl.accept_printable(b'3');
        repl.enter();
        repl.accept_printable(b'+');
        repl.enter();
        repl.accept_printable(b'.');
        repl.enter();

        assert_eq!(&repl.line_output()[..1], b"5");
    }

    #[test]
    fn handles_stack_words() {
        let mut repl = Repl::new();
        for bytes in [b"2".as_slice(), b"3", b"over", b"*", b"swap"] {
            for byte in bytes {
                repl.accept_printable(*byte);
            }
            repl.enter();
        }
        repl.accept_printable(b'.');
        repl.enter();

        assert_eq!(&repl.line_output()[..1], b"2");
        assert_eq!(repl.line_stack()[..5], [b'S', b':', b'<', b'1', b'>']);
    }

    #[test]
    fn prints_stack_summary() {
        let mut repl = Repl::new();
        for byte in b"1" {
            repl.accept_printable(*byte);
        }
        repl.enter();
        for byte in b"2" {
            repl.accept_printable(*byte);
        }
        repl.enter();
        for byte in b".s" {
            repl.accept_printable(*byte);
        }
        repl.enter();

        assert_eq!(&repl.line_output()[..7], b"S:<2> 1");
    }

    #[test]
    fn reports_underflow() {
        let mut repl = Repl::new();
        repl.accept_printable(b'+');
        repl.enter();

        assert_eq!(&repl.line_output()[..15], b"stack underflow");
    }

    #[test]
    fn reports_divide_by_zero() {
        let mut repl = Repl::new();
        for bytes in [b"4".as_slice(), b"0", b"/"] {
            for byte in bytes {
                repl.accept_printable(*byte);
            }
            repl.enter();
        }

        assert_eq!(&repl.line_output()[..14], b"divide by zero");
    }

    #[test]
    fn reports_unknown_word() {
        let mut repl = Repl::new();
        for byte in b"wat" {
            repl.accept_printable(*byte);
        }
        repl.enter();

        assert_eq!(&repl.line_output()[..12], b"unknown word");
    }

    #[test]
    fn keeps_prompt_scrolled_to_latest_input() {
        let mut repl = Repl::new();
        for _ in 0..30 {
            repl.accept_printable(b'x');
        }

        assert!(repl.line_prompt()[2..].iter().all(|byte| *byte == b'x'));
    }

    #[test]
    fn reports_input_full_without_growing_buffer() {
        let mut repl = Repl::new();
        for _ in 0..=INPUT_CAPACITY {
            repl.accept_printable(b'x');
        }

        assert_eq!(&repl.line_output()[..10], b"input full");
    }

    #[test]
    fn parses_and_prints_i32_min() {
        let mut repl = Repl::new();
        for bytes in [b"-2147483648".as_slice(), b"."] {
            for byte in bytes {
                repl.accept_printable(*byte);
            }
            repl.enter();
        }

        assert_eq!(&repl.line_output()[..11], b"-2147483648");
    }
}
