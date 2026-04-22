pub const INPUT_CAPACITY: usize = 64;
pub const LINE_WIDTH: usize = 28;
pub const OUTPUT_LINES: usize = 3;

const STACK_CAPACITY: usize = 16;

#[derive(Clone, Copy, Eq, PartialEq)]
enum EvalError {
    DivideByZero,
    StackOverflow,
    StackUnderflow,
    UnknownWord,
}

#[repr(C)]
#[derive(Clone, Eq, PartialEq)]
pub struct Repl {
    stack: [i32; STACK_CAPACITY],
    depth: usize,
    pub input: [u8; INPUT_CAPACITY],
    pub input_len: usize,
    pub output: [[u8; LINE_WIDTH]; OUTPUT_LINES],
}

impl Repl {
    pub const fn new() -> Self {
        Self {
            stack: [0; STACK_CAPACITY],
            depth: 0,
            input: [0; INPUT_CAPACITY],
            input_len: 0,
            output: [[b' '; LINE_WIDTH]; OUTPUT_LINES],
        }
    }

    pub fn push_byte(&mut self, byte: u8) {
        if self.input_len < INPUT_CAPACITY {
            input_set(&mut self.input, self.input_len, byte);
            self.input_len += 1;
        } else {
            self.push_input_full();
        }
    }

    pub fn backspace(&mut self) {
        self.input_len = self.input_len.saturating_sub(1);
    }

    pub fn eval_line(&mut self) {
        let mut index = 0;
        while let Some(token) = next_token_copy(&self.input, self.input_len, &mut index) {
            if let Err(error) = self.eval_token(&token) {
                self.push_error(error);
                self.input_len = 0;
                return;
            }
        }
        self.input_len = 0;
    }

    pub fn stack_line(&self) -> [u8; LINE_WIDTH] {
        let mut line = [b' '; LINE_WIDTH];
        line[0] = b'S';
        line[1] = b':';
        line[2] = b'<';
        let mut offset = write_usize(&mut line, 3, self.depth);
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
            offset = write_i32(&mut line, offset, stack_get(&self.stack, index));
            if offset < LINE_WIDTH {
                line_set(&mut line, offset, b' ');
                offset += 1;
            }
            index += 1;
        }
        line
    }

    pub fn prompt_line(&self) -> [u8; LINE_WIDTH] {
        let mut line = [b' '; LINE_WIDTH];
        line[0] = b'>';
        line[1] = b' ';
        let visible = self.input_len.min(LINE_WIDTH - 2);
        let start = self.input_len - visible;
        let mut index = 0;
        while index < visible {
            line_set(&mut line, index + 2, input_get(&self.input, start + index));
            index += 1;
        }
        line
    }

    fn eval_token(&mut self, token: &Token) -> Result<(), EvalError> {
        if let Some(number) = parse_i32(token) {
            return self.push(number);
        }
        if token_eq(token, *b"+") {
            self.binary_op(i32::wrapping_add)
        } else if token_eq(token, *b"-") {
            self.binary_op(i32::wrapping_sub)
        } else if token_eq(token, *b"*") {
            self.binary_op(i32::wrapping_mul)
        } else if token_eq(token, *b"/") {
            self.checked_div()
        } else if token_eq(token, *b"mod") {
            self.checked_mod()
        } else if token_eq(token, *b"dup") {
            self.dup()
        } else if token_eq(token, *b"drop") {
            self.drop()
        } else if token_eq(token, *b"swap") {
            self.swap()
        } else if token_eq(token, *b"over") {
            self.over()
        } else if token_eq(token, *b".") {
            self.print_top()
        } else if token_eq(token, *b".s") {
            self.print_stack();
            Ok(())
        } else if token_eq(token, *b"clear") {
            self.depth = 0;
            self.push_ok();
            Ok(())
        } else {
            Err(EvalError::UnknownWord)
        }
    }

    fn push(&mut self, value: i32) -> Result<(), EvalError> {
        if self.depth == STACK_CAPACITY {
            return Err(EvalError::StackOverflow);
        }
        stack_set(&mut self.stack, self.depth, value);
        self.depth += 1;
        Ok(())
    }

    fn pop(&mut self) -> Result<i32, EvalError> {
        if self.depth == 0 {
            return Err(EvalError::StackUnderflow);
        }
        self.depth -= 1;
        Ok(stack_get(&self.stack, self.depth))
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
        self.push(lhs.wrapping_div(rhs))
    }

    fn checked_mod(&mut self) -> Result<(), EvalError> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        if rhs == 0 {
            return Err(EvalError::DivideByZero);
        }
        self.push(lhs.wrapping_rem(rhs))
    }

    fn dup(&mut self) -> Result<(), EvalError> {
        if self.depth == 0 {
            return Err(EvalError::StackUnderflow);
        }
        let value = stack_get(&self.stack, self.depth - 1);
        self.push(value)
    }

    fn drop(&mut self) -> Result<(), EvalError> {
        self.pop().map(|_| ())
    }

    fn swap(&mut self) -> Result<(), EvalError> {
        if self.depth < 2 {
            return Err(EvalError::StackUnderflow);
        }
        let top = stack_get(&self.stack, self.depth - 1);
        let next = stack_get(&self.stack, self.depth - 2);
        stack_set(&mut self.stack, self.depth - 1, next);
        stack_set(&mut self.stack, self.depth - 2, top);
        Ok(())
    }

    fn over(&mut self) -> Result<(), EvalError> {
        if self.depth < 2 {
            return Err(EvalError::StackUnderflow);
        }
        self.push(stack_get(&self.stack, self.depth - 2))
    }

    fn print_top(&mut self) -> Result<(), EvalError> {
        let value = self.pop()?;
        self.push_number_message(value);
        Ok(())
    }

    fn print_stack(&mut self) {
        let mut line = [b' '; LINE_WIDTH];
        line[0] = b'<';
        let mut offset = write_usize(&mut line, 1, self.depth);
        if offset < LINE_WIDTH {
            line[offset] = b'>';
            offset += 1;
        }
        if offset < LINE_WIDTH {
            line[offset] = b' ';
            offset += 1;
        }
        let mut index = 0;
        while index < self.depth && offset < LINE_WIDTH {
            offset = write_i32(&mut line, offset, stack_get(&self.stack, index));
            if offset < LINE_WIDTH {
                line_set(&mut line, offset, b' ');
                offset += 1;
            }
            index += 1;
        }
        self.push_line(line);
    }

    fn push_number_message(&mut self, value: i32) {
        let mut line = [b' '; LINE_WIDTH];
        write_i32(&mut line, 0, value);
        self.push_line(line);
    }

    fn push_error(&mut self, error: EvalError) {
        let mut line = [b' '; LINE_WIDTH];
        match error {
            EvalError::DivideByZero => write_divide_by_zero(&mut line),
            EvalError::StackOverflow => write_stack_overflow(&mut line),
            EvalError::StackUnderflow => write_stack_underflow(&mut line),
            EvalError::UnknownWord => write_unknown_word(&mut line),
        }
        self.push_line(line);
    }

    fn push_input_full(&mut self) {
        let mut line = [b' '; LINE_WIDTH];
        write_input_full(&mut line);
        self.push_line(line);
    }

    fn push_ok(&mut self) {
        let mut line = [b' '; LINE_WIDTH];
        line_set(&mut line, 0, b'o');
        line_set(&mut line, 1, b'k');
        self.push_line(line);
    }

    fn push_line(&mut self, line: [u8; LINE_WIDTH]) {
        self.output[0] = self.output[1];
        self.output[1] = self.output[2];
        self.output[2] = line;
    }
}

struct Token {
    bytes: [u8; INPUT_CAPACITY],
    len: usize,
}

fn token_eq<const N: usize>(token: &Token, expected: [u8; N]) -> bool {
    if token.len != N {
        return false;
    }
    let mut index = 0;
    while index < N {
        if input_get(&token.bytes, index) != array_get(&expected, index) {
            return false;
        }
        index += 1;
    }
    true
}

fn next_token_copy(
    input: &[u8; INPUT_CAPACITY],
    input_len: usize,
    index: &mut usize,
) -> Option<Token> {
    while *index < input_len && input_get(input, *index) == b' ' {
        *index += 1;
    }
    let start = *index;
    while *index < input_len && input_get(input, *index) != b' ' {
        *index += 1;
    }
    if start == *index {
        return None;
    }
    let mut token = Token {
        bytes: [0; INPUT_CAPACITY],
        len: 0,
    };
    while start + token.len < *index && token.len < INPUT_CAPACITY {
        input_set(
            &mut token.bytes,
            token.len,
            input_get(input, start + token.len),
        );
        token.len += 1;
    }
    Some(token)
}

fn parse_i32(token: &Token) -> Option<i32> {
    if token.len == 0 {
        return None;
    }
    let negative = input_get(&token.bytes, 0) == b'-';
    let mut index = usize::from(negative);
    if index == token.len {
        return None;
    }
    let mut value = 0_i32;
    while index < token.len {
        let byte = input_get(&token.bytes, index);
        if !byte.is_ascii_digit() {
            return None;
        }
        value = checked_mul10(value)?;
        value = value.checked_add(i32::from(byte - b'0'))?;
        index += 1;
    }
    Some(if negative { -value } else { value })
}

fn checked_mul10(value: i32) -> Option<i32> {
    let by_eight = value.checked_shl(3)?;
    let by_two = value.checked_shl(1)?;
    by_eight.checked_add(by_two)
}

fn write_usize(target: &mut [u8; LINE_WIDTH], offset: usize, value: usize) -> usize {
    let Ok(value) = i32::try_from(value) else {
        return offset;
    };
    write_i32(target, offset, value)
}

fn write_i32(target: &mut [u8; LINE_WIDTH], mut offset: usize, value: i32) -> usize {
    if offset >= LINE_WIDTH {
        return offset;
    }
    if value == 0 {
        line_set(target, offset, b'0');
        return offset + 1;
    }
    let mut work = value;
    if work < 0 {
        line_set(target, offset, b'-');
        offset += 1;
        work = work.saturating_abs();
    }
    let mut digits = [0_u8; 10];
    let mut count = 0;
    while work > 0 && count < 10 {
        let digit = u8::try_from(work.wrapping_rem(10)).unwrap_or_default();
        digit_set(&mut digits, count, b'0' + digit);
        work = work.wrapping_div(10);
        count += 1;
    }
    while count > 0 && offset < LINE_WIDTH {
        count -= 1;
        line_set(target, offset, digit_get(&digits, count));
        offset += 1;
    }
    offset
}

fn write_divide_by_zero(line: &mut [u8; LINE_WIDTH]) {
    line_set(line, 0, b'd');
    line_set(line, 1, b'i');
    line_set(line, 2, b'v');
    line_set(line, 3, b'i');
    line_set(line, 4, b'd');
    line_set(line, 5, b'e');
    line_set(line, 6, b' ');
    line_set(line, 7, b'b');
    line_set(line, 8, b'y');
    line_set(line, 9, b' ');
    line_set(line, 10, b'z');
    line_set(line, 11, b'e');
    line_set(line, 12, b'r');
    line_set(line, 13, b'o');
}

fn write_stack_overflow(line: &mut [u8; LINE_WIDTH]) {
    line_set(line, 0, b's');
    line_set(line, 1, b't');
    line_set(line, 2, b'a');
    line_set(line, 3, b'c');
    line_set(line, 4, b'k');
    line_set(line, 5, b' ');
    line_set(line, 6, b'o');
    line_set(line, 7, b'v');
    line_set(line, 8, b'e');
    line_set(line, 9, b'r');
    line_set(line, 10, b'f');
    line_set(line, 11, b'l');
    line_set(line, 12, b'o');
    line_set(line, 13, b'w');
}

fn write_stack_underflow(line: &mut [u8; LINE_WIDTH]) {
    line_set(line, 0, b's');
    line_set(line, 1, b't');
    line_set(line, 2, b'a');
    line_set(line, 3, b'c');
    line_set(line, 4, b'k');
    line_set(line, 5, b' ');
    line_set(line, 6, b'u');
    line_set(line, 7, b'n');
    line_set(line, 8, b'd');
    line_set(line, 9, b'e');
    line_set(line, 10, b'r');
    line_set(line, 11, b'f');
    line_set(line, 12, b'l');
    line_set(line, 13, b'o');
    line_set(line, 14, b'w');
}

fn write_unknown_word(line: &mut [u8; LINE_WIDTH]) {
    line_set(line, 0, b'u');
    line_set(line, 1, b'n');
    line_set(line, 2, b'k');
    line_set(line, 3, b'n');
    line_set(line, 4, b'o');
    line_set(line, 5, b'w');
    line_set(line, 6, b'n');
    line_set(line, 7, b' ');
    line_set(line, 8, b'w');
    line_set(line, 9, b'o');
    line_set(line, 10, b'r');
    line_set(line, 11, b'd');
}

fn write_input_full(line: &mut [u8; LINE_WIDTH]) {
    line_set(line, 0, b'i');
    line_set(line, 1, b'n');
    line_set(line, 2, b'p');
    line_set(line, 3, b'u');
    line_set(line, 4, b't');
    line_set(line, 5, b' ');
    line_set(line, 6, b'f');
    line_set(line, 7, b'u');
    line_set(line, 8, b'l');
    line_set(line, 9, b'l');
}

fn input_get(input: &[u8; INPUT_CAPACITY], index: usize) -> u8 {
    debug_assert!(index < INPUT_CAPACITY);
    // SAFETY: Every call site guards index against INPUT_CAPACITY or a smaller live length.
    unsafe { *input.get_unchecked(index) }
}

fn input_set(input: &mut [u8; INPUT_CAPACITY], index: usize, value: u8) {
    debug_assert!(index < INPUT_CAPACITY);
    // SAFETY: Every call site guards index against INPUT_CAPACITY or a smaller live length.
    unsafe {
        *input.get_unchecked_mut(index) = value;
    }
}

fn stack_get(stack: &[i32; STACK_CAPACITY], index: usize) -> i32 {
    debug_assert!(index < STACK_CAPACITY);
    // SAFETY: Stack depth is capped at STACK_CAPACITY, and callers guard underflow.
    unsafe { *stack.get_unchecked(index) }
}

fn stack_set(stack: &mut [i32; STACK_CAPACITY], index: usize, value: i32) {
    debug_assert!(index < STACK_CAPACITY);
    // SAFETY: Stack depth is capped at STACK_CAPACITY before writes.
    unsafe {
        *stack.get_unchecked_mut(index) = value;
    }
}

fn line_set(line: &mut [u8; LINE_WIDTH], index: usize, value: u8) {
    debug_assert!(index < LINE_WIDTH);
    // SAFETY: Callers only write when index is below LINE_WIDTH.
    unsafe {
        *line.get_unchecked_mut(index) = value;
    }
}

fn digit_get(digits: &[u8; 10], index: usize) -> u8 {
    debug_assert!(index < 10);
    // SAFETY: Decimal i32 formatting never uses more than 10 stored digits.
    unsafe { *digits.get_unchecked(index) }
}

fn digit_set(digits: &mut [u8; 10], index: usize, value: u8) {
    debug_assert!(index < 10);
    // SAFETY: Decimal i32 formatting never stores more than 10 digits.
    unsafe {
        *digits.get_unchecked_mut(index) = value;
    }
}

fn array_get<const N: usize>(array: &[u8; N], index: usize) -> u8 {
    debug_assert!(index < N);
    // SAFETY: Generic callers compare index with N before reading.
    unsafe { *array.get_unchecked(index) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_arithmetic_and_prints() {
        let mut repl = Repl::new();
        repl.input[..7].copy_from_slice(b"2 3 + .");
        repl.input_len = 7;
        repl.eval_line();
        assert_eq!(&repl.output[2][..1], b"5");
        assert_eq!(repl.depth, 0);
    }

    #[test]
    fn handles_stack_words() {
        let mut repl = Repl::new();
        repl.input[..17].copy_from_slice(b"2 3 over * swap .");
        repl.input_len = 17;
        repl.eval_line();
        assert_eq!(&repl.output[2][..1], b"2");
        assert_eq!(repl.stack[..repl.depth], [6]);
    }

    #[test]
    fn prints_stack_summary() {
        let mut repl = Repl::new();
        repl.input[..6].copy_from_slice(b"1 2 .s");
        repl.input_len = 6;
        repl.eval_line();
        assert_eq!(&repl.output[2][..7], b"<2> 1 2");
    }

    #[test]
    fn reports_underflow() {
        let mut repl = Repl::new();
        repl.input[0] = b'+';
        repl.input_len = 1;
        repl.eval_line();
        assert_eq!(&repl.output[2][..15], b"stack underflow");
    }

    #[test]
    fn reports_divide_by_zero() {
        let mut repl = Repl::new();
        repl.input[..5].copy_from_slice(b"4 0 /");
        repl.input_len = 5;
        repl.eval_line();
        assert_eq!(&repl.output[2][..14], b"divide by zero");
    }

    #[test]
    fn reports_unknown_word() {
        let mut repl = Repl::new();
        repl.input[..3].copy_from_slice(b"wat");
        repl.input_len = 3;
        repl.eval_line();
        assert_eq!(&repl.output[2][..12], b"unknown word");
    }
}
