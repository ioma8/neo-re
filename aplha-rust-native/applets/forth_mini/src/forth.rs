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
    pending_output_kind: u32,
    pending_output_value: i32,
    transcript_1: [u8; LINE_WIDTH],
    transcript_2: [u8; LINE_WIDTH],
    transcript_3: [u8; LINE_WIDTH],
}

impl Repl {
    pub const fn new() -> Self {
        Self {
            stack: [0; STACK_CAPACITY],
            depth: 0,
            input: [0; INPUT_CAPACITY],
            input_len: 0,
            pending_output_kind: 0,
            pending_output_value: 0,
            transcript_1: blank_line(),
            transcript_2: blank_line(),
            transcript_3: blank_line(),
        }
    }

    pub fn accept_printable(&mut self, byte: u8) {
        if !(b' '..=b'~').contains(&byte) {
            return;
        }
        if self.input_len >= INPUT_CAPACITY {
            return;
        }
        input_set(&mut self.input, self.input_len, byte);
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

        shift_transcripts(self);
        clear_line(&mut self.transcript_3);
        self.pending_output_kind = 0;
        self.pending_output_value = 0;
        let mut cursor = 0usize;
        while let Some((start, end)) = next_token_bounds(&self.input, self.input_len, &mut cursor) {
            if let Err(error) = self.eval_token(start, end) {
                    let command_len = input_trimmed_len(&self.input, self.input_len);
                    self.write_error_transcript(command_len, error);
                    self.input_len = 0;
                    return;
            }
        }
        let command_len = input_trimmed_len(&self.input, self.input_len);
        self.write_success_transcript(command_len);
        self.input_len = 0;
    }

    pub fn line_prompt(&self) -> [u8; LINE_WIDTH] {
        let mut line = blank_line();
        let visible = self.input_len.min(LINE_WIDTH);
        let mut index = 0usize;
        while index < visible {
            line_set(&mut line, index, input_get(&self.input, index));
            index += 1;
        }
        line
    }

    pub fn line_transcript(&self, index: usize) -> [u8; LINE_WIDTH] {
        debug_assert!(index < OUTPUT_LINES);
        match index {
            0 => self.transcript_1,
            1 => self.transcript_2,
            _ => self.transcript_3,
        }
    }

    #[cfg(test)]
    pub fn line_output(&self) -> [u8; LINE_WIDTH] {
        self.transcript_3
    }

    #[cfg(test)]
    pub fn line_stack(&self) -> [u8; LINE_WIDTH] {
        self.stack_summary_line()
    }

    fn eval_token(&mut self, start: usize, end: usize) -> Result<(), EvalError> {
        if let Some(value) = parse_i32_slice(&self.input, start, end) {
            self.push(value)?;
            return Ok(());
        }

        if token_eq_slice(&self.input, start, end, b"+") || token_eq_slice(&self.input, start, end, b"=") {
            self.add()?;
            Ok(())
        } else if token_eq_slice(&self.input, start, end, b"-") {
            self.sub()?;
            Ok(())
        } else if token_eq_slice(&self.input, start, end, b"*") {
            self.mul()?;
            Ok(())
        } else if token_eq_slice(&self.input, start, end, b"/") {
            self.checked_div()?;
            Ok(())
        } else if token_eq_slice(&self.input, start, end, b"mod") {
            self.checked_mod()?;
            Ok(())
        } else if token_eq_slice(&self.input, start, end, b"dup") {
            self.dup()?;
            Ok(())
        } else if token_eq_slice(&self.input, start, end, b"drop") {
            self.drop_top()?;
            Ok(())
        } else if token_eq_slice(&self.input, start, end, b"swap") {
            self.swap()?;
            Ok(())
        } else if token_eq_slice(&self.input, start, end, b"over") {
            self.over()?;
            Ok(())
        } else if token_eq_slice(&self.input, start, end, b".") {
            self.pop_to_pending_output()
        } else if token_eq_slice(&self.input, start, end, b".s") {
            self.pending_output_kind = 2;
            Ok(())
        } else if token_eq_slice(&self.input, start, end, b"clear") {
            self.depth = 0;
            Ok(())
        } else {
            Err(EvalError::UnknownWord)
        }
    }

    fn write_success_transcript(&mut self, command_len: usize) {
        let mut offset = self.write_command_prefix(command_len);
        match self.pending_output_kind {
            0 => {}
            1 => {
                if offset < LINE_WIDTH {
                    line_set(&mut self.transcript_3, offset, b' ');
                    offset += 1;
                }
                let _ = write_i32(&mut self.transcript_3, offset, self.pending_output_value);
                offset = trimmed_len(&self.transcript_3);
            }
            2 => {
                if offset < LINE_WIDTH {
                    line_set(&mut self.transcript_3, offset, b' ');
                    offset += 1;
                }
                let _ = write_stack_summary_from(&self.stack, self.depth, &mut self.transcript_3, offset);
                offset = trimmed_len(&self.transcript_3);
            }
            _ => {}
        }
        if offset < LINE_WIDTH {
            line_set(&mut self.transcript_3, offset, b' ');
            offset += 1;
        }
        if offset < LINE_WIDTH {
            line_set(&mut self.transcript_3, offset, b' ');
            offset += 1;
        }
        if offset < LINE_WIDTH {
            line_set(&mut self.transcript_3, offset, b'o');
            offset += 1;
        }
        if offset < LINE_WIDTH {
            line_set(&mut self.transcript_3, offset, b'k');
        }
    }

    fn write_error_transcript(&mut self, command_len: usize, error: EvalError) {
        let mut offset = self.write_command_prefix(command_len);
        if offset < LINE_WIDTH {
            line_set(&mut self.transcript_3, offset, b' ');
            offset += 1;
        }
        match error {
            EvalError::DivideByZero => write_divide_by_zero_at(&mut self.transcript_3, offset),
            EvalError::InputFull => write_input_full_at(&mut self.transcript_3, offset),
            EvalError::StackOverflow => write_stack_overflow_at(&mut self.transcript_3, offset),
            EvalError::StackUnderflow => write_stack_underflow_at(&mut self.transcript_3, offset),
            EvalError::UnknownWord => write_unknown_word_at(&mut self.transcript_3, offset),
        }
    }

    fn write_command_prefix(&mut self, command_len: usize) -> usize {
        let command_visible = command_len.min(LINE_WIDTH);
        let mut offset = 0usize;
        while offset < command_visible {
            line_set(&mut self.transcript_3, offset, input_get(&self.input, offset));
            offset += 1;
        }
        offset
    }

    fn push(&mut self, value: i32) -> Result<(), EvalError> {
        if self.depth >= STACK_CAPACITY {
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

    fn add(&mut self) -> Result<(), EvalError> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        self.push(lhs.wrapping_add(rhs))
    }

    fn sub(&mut self) -> Result<(), EvalError> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        self.push(lhs.wrapping_sub(rhs))
    }

    fn mul(&mut self) -> Result<(), EvalError> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        self.push(mul_i32_wrapping(lhs, rhs))
    }

    fn checked_div(&mut self) -> Result<(), EvalError> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        if rhs == 0 {
            return Err(EvalError::DivideByZero);
        }
        self.push(div_i32_wrapping(lhs, rhs))
    }

    fn checked_mod(&mut self) -> Result<(), EvalError> {
        let rhs = self.pop()?;
        let lhs = self.pop()?;
        if rhs == 0 {
            return Err(EvalError::DivideByZero);
        }
        self.push(rem_i32_wrapping(lhs, rhs))
    }

    fn dup(&mut self) -> Result<(), EvalError> {
        if self.depth == 0 {
            return Err(EvalError::StackUnderflow);
        }
        self.push(stack_get(&self.stack, self.depth - 1))
    }

    fn drop_top(&mut self) -> Result<(), EvalError> {
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

    fn pop_to_pending_output(&mut self) -> Result<(), EvalError> {
        if self.depth == 0 {
            return Err(EvalError::StackUnderflow);
        }
        self.pending_output_value = self.pop()?;
        self.pending_output_kind = 1;
        Ok(())
    }

    fn stack_summary_line(&self) -> [u8; LINE_WIDTH] {
        let mut line = blank_line();
        write_stack_summary_from(&self.stack, self.depth, &mut line, 0);
        line
    }
}

fn write_stack_summary_from(
    stack: &[i32; STACK_CAPACITY],
    depth: usize,
    line: &mut [u8; LINE_WIDTH],
    offset: usize,
) -> usize {
    let mut next = offset;
    if next < LINE_WIDTH {
        line_set(line, next, b'<');
        next += 1;
    }
    let _ = write_usize(line, next, depth);
    next = trimmed_len(line);
    if next < LINE_WIDTH {
        line_set(line, next, b'>');
        next += 1;
    }
    if next < LINE_WIDTH {
        line_set(line, next, b' ');
        next += 1;
    }
    let start = depth.saturating_sub(3);
    let mut index = start;
    while index < depth && next < LINE_WIDTH {
        let _ = write_i32(line, next, stack_get(stack, index));
        next = trimmed_len(line);
        if next < LINE_WIDTH && index + 1 < depth {
            line_set(line, next, b' ');
            next += 1;
        }
        index += 1;
    }
    next
}

const fn blank_line() -> [u8; LINE_WIDTH] {
    [b' '; LINE_WIDTH]
}

fn write_stack_prefix(line: &mut [u8; LINE_WIDTH], depth: usize) -> usize {
    line_set(line, 0, b'<');
    let mut offset = write_usize(line, 1, depth);
    if offset < LINE_WIDTH {
        line_set(line, offset, b'>');
        offset += 1;
    }
    if offset < LINE_WIDTH {
        line_set(line, offset, b' ');
        offset += 1;
    }
    offset
}

fn trimmed_len(line: &[u8; LINE_WIDTH]) -> usize {
    let mut len = LINE_WIDTH;
    while len > 0 && is_blank(line_get(line, len - 1)) {
        len -= 1;
    }
    len
}

fn is_blank(byte: u8) -> bool {
    byte == b' ' || byte == 0
}

fn clear_line(line: &mut [u8; LINE_WIDTH]) {
    let mut index = 0usize;
    while index < LINE_WIDTH {
        line_set(line, index, b' ');
        index += 1;
    }
}

fn copy_line(dst: &mut [u8; LINE_WIDTH], src: &[u8; LINE_WIDTH]) {
    let mut index = 0usize;
    while index < LINE_WIDTH {
        line_set(dst, index, line_get(src, index));
        index += 1;
    }
}

fn shift_transcripts(repl: &mut Repl) {
    let mut index = 0usize;
    while index < LINE_WIDTH {
        let value = line_get(&repl.transcript_2, index);
        line_set(&mut repl.transcript_1, index, value);
        index += 1;
    }
    let mut index = 0usize;
    while index < LINE_WIDTH {
        let value = line_get(&repl.transcript_3, index);
        line_set(&mut repl.transcript_2, index, value);
        index += 1;
    }
}

fn token_eq_slice(input: &[u8; INPUT_CAPACITY], start: usize, end: usize, expected: &[u8]) -> bool {
    if end.saturating_sub(start) != expected.len() {
        return false;
    }
    let mut index = 0usize;
    while index < expected.len() {
        if input_get(input, start + index) != expected[index] {
            return false;
        }
        index += 1;
    }
    true
}

fn next_token_bounds(
    input: &[u8; INPUT_CAPACITY],
    input_len: usize,
    cursor: &mut usize,
) -> Option<(usize, usize)> {
    while *cursor < input_len && input_get(input, *cursor) == b' ' {
        *cursor += 1;
    }
    let start = *cursor;
    while *cursor < input_len && input_get(input, *cursor) != b' ' {
        *cursor += 1;
    }
    if start == *cursor {
        return None;
    }
    Some((start, *cursor))
}

fn parse_i32_slice(input: &[u8; INPUT_CAPACITY], start: usize, end: usize) -> Option<i32> {
    if start >= end {
        return None;
    }

    let negative = input_get(input, start) == b'-';
    let mut index = start + usize::from(negative);
    if index == end {
        return None;
    }

    let limit = if negative {
        i32::MAX as u32 + 1
    } else {
        i32::MAX as u32
    };
    let mut magnitude = 0u32;
    while index < end {
        let byte = input_get(input, index);
        if !byte.is_ascii_digit() {
            return None;
        }
        let digit = u32::from(byte - b'0');
        if magnitude > limit / 10 || (magnitude == limit / 10 && digit > limit % 10) {
            return None;
        }
        magnitude = magnitude * 10 + digit;
        index += 1;
    }

    if negative {
        if magnitude == i32::MAX as u32 + 1 {
            Some(i32::MIN)
        } else {
            Some(-(magnitude as i32))
        }
    } else {
        Some(magnitude as i32)
    }
}

fn input_trimmed_len(input: &[u8; INPUT_CAPACITY], input_len: usize) -> usize {
    let mut len = input_len.min(LINE_WIDTH);
    while len > 0 && input_get(input, len - 1) == b' ' {
        len -= 1;
    }
    len
}

fn write_i32(line: &mut [u8; LINE_WIDTH], mut offset: usize, value: i32) -> usize {
    if offset >= LINE_WIDTH {
        return offset;
    }

    if value == 0 {
        line_set(line, offset, b'0');
        return offset + 1;
    }
    if value < 0 {
        line_set(line, offset, b'-');
        offset += 1;
    }

    let mut remainder = i32_magnitude_u32(value);
    let mut divisor = 1_000_000_000u32;
    let mut started = false;
    while divisor != 0 && offset < LINE_WIDTH {
        let mut digit = 0u8;
        while remainder >= divisor {
            remainder = remainder.wrapping_sub(divisor);
            digit = digit.wrapping_add(1);
        }
        if started || digit != 0 || divisor == 1 {
            line_set(line, offset, b'0' + digit as u8);
            offset += 1;
            started = true;
        }
        divisor = match divisor {
            1_000_000_000 => 100_000_000,
            100_000_000 => 10_000_000,
            10_000_000 => 1_000_000,
            1_000_000 => 100_000,
            100_000 => 10_000,
            10_000 => 1_000,
            1_000 => 100,
            100 => 10,
            10 => 1,
            _ => 0,
        };
    }
    offset
}

fn write_usize(line: &mut [u8; LINE_WIDTH], mut offset: usize, mut value: usize) -> usize {
    if offset >= LINE_WIDTH {
        return offset;
    }

    let mut tens = 0u8;
    while value >= 10 {
        value -= 10;
        tens = tens.wrapping_add(1);
    }
    if tens != 0 {
        line_set(line, offset, b'0' + tens);
        offset += 1;
    }
    if offset < LINE_WIDTH {
        line_set(line, offset, b'0' + value as u8);
        offset += 1;
    }
    offset
}

fn i32_magnitude_u32(value: i32) -> u32 {
    if value < 0 {
        value.wrapping_neg() as u32
    } else {
        value as u32
    }
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

fn write_divide_by_zero_at(line: &mut [u8; LINE_WIDTH], offset: usize) {
    let mut text = blank_line();
    write_divide_by_zero(&mut text);
    copy_trimmed_line(line, offset, &text);
}

fn write_input_full_at(line: &mut [u8; LINE_WIDTH], offset: usize) {
    let mut text = blank_line();
    write_input_full(&mut text);
    copy_trimmed_line(line, offset, &text);
}

fn write_stack_overflow_at(line: &mut [u8; LINE_WIDTH], offset: usize) {
    let mut text = blank_line();
    write_stack_overflow(&mut text);
    copy_trimmed_line(line, offset, &text);
}

fn write_stack_underflow_at(line: &mut [u8; LINE_WIDTH], offset: usize) {
    let mut text = blank_line();
    write_stack_underflow(&mut text);
    copy_trimmed_line(line, offset, &text);
}

fn write_unknown_word_at(line: &mut [u8; LINE_WIDTH], offset: usize) {
    let mut text = blank_line();
    write_unknown_word(&mut text);
    copy_trimmed_line(line, offset, &text);
}

fn copy_trimmed_line(line: &mut [u8; LINE_WIDTH], mut offset: usize, text: &[u8; LINE_WIDTH]) {
    let mut index = 0usize;
    let len = trimmed_len(text);
    while index < len && offset < LINE_WIDTH {
        line_set(line, offset, line_get(text, index));
        index += 1;
        offset += 1;
    }
}

fn input_get(input: &[u8; INPUT_CAPACITY], index: usize) -> u8 {
    debug_assert!(index < INPUT_CAPACITY);
    unsafe { *input.get_unchecked(index) }
}

fn input_set(input: &mut [u8; INPUT_CAPACITY], index: usize, value: u8) {
    debug_assert!(index < INPUT_CAPACITY);
    unsafe {
        *input.get_unchecked_mut(index) = value;
    }
}

fn stack_get(stack: &[i32; STACK_CAPACITY], index: usize) -> i32 {
    debug_assert!(index < STACK_CAPACITY);
    unsafe { *stack.get_unchecked(index) }
}

fn stack_set(stack: &mut [i32; STACK_CAPACITY], index: usize, value: i32) {
    debug_assert!(index < STACK_CAPACITY);
    unsafe {
        *stack.get_unchecked_mut(index) = value;
    }
}

fn line_set(line: &mut [u8; LINE_WIDTH], index: usize, value: u8) {
    debug_assert!(index < LINE_WIDTH);
    #[cfg(target_arch = "m68k")]
    unsafe {
        core::ptr::write_volatile(line.get_unchecked_mut(index), value);
    }
    #[cfg(not(target_arch = "m68k"))]
    unsafe {
        *line.get_unchecked_mut(index) = value;
    }
}

fn line_get(line: &[u8; LINE_WIDTH], index: usize) -> u8 {
    debug_assert!(index < LINE_WIDTH);
    #[cfg(target_arch = "m68k")]
    unsafe {
        core::ptr::read_volatile(line.get_unchecked(index))
    }
    #[cfg(not(target_arch = "m68k"))]
    unsafe {
        *line.get_unchecked(index)
    }
}

fn udivmod32_local(numerator: u32, denominator: u32) -> (u32, u32) {
    if denominator == 0 {
        return (0, numerator);
    }
    let mut quotient = 0u32;
    let mut remainder = 0u32;
    let mut bit = 32u32;
    while bit != 0 {
        bit -= 1;
        remainder = remainder.wrapping_shl(1);
        remainder |= (numerator >> bit) & 1;
        if remainder >= denominator {
            remainder = remainder.wrapping_sub(denominator);
            quotient |= 1u32 << bit;
        }
    }
    (quotient, remainder)
}


fn mul_i32_wrapping(lhs: i32, rhs: i32) -> i32 {
    let negative = (lhs < 0) ^ (rhs < 0);
    let mut a = lhs.unsigned_abs();
    let mut b = rhs.unsigned_abs();
    let mut result = 0_u32;
    while b != 0 {
        if b & 1 != 0 {
            result = result.wrapping_add(a);
        }
        a = a.wrapping_shl(1);
        b >>= 1;
    }
    if negative {
        result.wrapping_neg().cast_signed()
    } else {
        result.cast_signed()
    }
}

fn div_i32_wrapping(lhs: i32, rhs: i32) -> i32 {
    let negative = (lhs < 0) ^ (rhs < 0);
    let (quotient, _) = udivmod32_local(lhs.unsigned_abs(), rhs.unsigned_abs());
    if negative {
        quotient.wrapping_neg().cast_signed()
    } else {
        quotient.cast_signed()
    }
}

fn rem_i32_wrapping(lhs: i32, rhs: i32) -> i32 {
    let negative = lhs < 0;
    let (_, remainder) = udivmod32_local(lhs.unsigned_abs(), rhs.unsigned_abs());
    if negative {
        remainder.wrapping_neg().cast_signed()
    } else {
        remainder.cast_signed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn last_output(repl: &Repl) -> [u8; LINE_WIDTH] {
        repl.line_output()
    }

    #[test]
    fn accepts_printable_without_evaluating_until_enter() {
        let mut repl = Repl::new();

        repl.accept_printable(b'1');

        assert_eq!(last_output(&repl), blank_line());
        assert_eq!(repl.line_prompt()[..3], [b'1', b' ', b' ']);
    }

    #[test]
    fn enter_evaluates_current_input_line() {
        let mut repl = Repl::new();

        repl.accept_printable(b'1');
        repl.enter();

        assert_eq!(&last_output(&repl)[..5], b"1  ok");
    }

    #[test]
    fn backspace_removes_one_input_byte() {
        let mut repl = Repl::new();

        repl.accept_printable(b'1');
        repl.accept_printable(b'2');
        repl.backspace();

        assert_eq!(repl.line_prompt()[..3], [b'1', b' ', b' ']);
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

        assert_eq!(&last_output(&repl)[..7], b". 5  ok");
    }

    #[test]
    fn evaluates_subtraction_and_prints() {
        let mut repl = Repl::new();
        repl.accept_printable(b'2');
        repl.enter();
        repl.accept_printable(b'1');
        repl.enter();
        repl.accept_printable(b'-');
        repl.enter();
        repl.accept_printable(b'.');
        repl.enter();

        assert_eq!(&last_output(&repl)[..7], b". 1  ok");
    }

    #[test]
    fn evaluates_division_and_prints() {
        let mut repl = Repl::new();
        repl.accept_printable(b'8');
        repl.enter();
        repl.accept_printable(b'2');
        repl.enter();
        repl.accept_printable(b'/');
        repl.enter();
        repl.accept_printable(b'.');
        repl.enter();

        assert_eq!(&last_output(&repl)[..7], b". 4  ok");
    }

    #[test]
    fn evaluates_mod_and_prints() {
        let mut repl = Repl::new();
        repl.accept_printable(b'8');
        repl.enter();
        repl.accept_printable(b'3');
        repl.enter();
        for byte in b"mod" {
            repl.accept_printable(*byte);
        }
        repl.enter();
        repl.accept_printable(b'.');
        repl.enter();

        assert_eq!(&last_output(&repl)[..7], b". 2  ok");
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

        assert_eq!(&last_output(&repl)[..7], b". 2  ok");
        assert_eq!(&repl.line_stack()[..5], b"<1> 6");
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

        assert_eq!(&last_output(&repl)[..14], b".s <2> 1 2  ok");
    }

    #[test]
    fn dot_preserves_popped_value_until_transcript_render() {
        let mut repl = Repl::new();
        for byte in b"1 . 2" {
            repl.accept_printable(*byte);
        }
        repl.enter();

        assert_eq!(&last_output(&repl)[..8], b"1 . 2 1 ");
        assert_eq!(&repl.line_stack()[..5], b"<1> 2");
    }

    #[test]
    fn reports_underflow() {
        let mut repl = Repl::new();
        repl.accept_printable(b'+');
        repl.enter();

        assert_eq!(&last_output(&repl)[..17], b"+ stack underflow");
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

        assert_eq!(&last_output(&repl)[..16], b"/ divide by zero");
    }

    #[test]
    fn reports_unknown_word() {
        let mut repl = Repl::new();
        for byte in b"wat" {
            repl.accept_printable(*byte);
        }
        repl.enter();

        assert_eq!(&last_output(&repl)[..16], b"wat unknown word");
    }

    #[test]
    fn keeps_prompt_scrolled_to_latest_input() {
        let mut repl = Repl::new();
        for _ in 0..30 {
            repl.accept_printable(b'x');
        }

        assert!(repl.line_prompt().iter().all(|byte| *byte == b'x'));
    }

    #[test]
    fn reports_input_full_without_growing_buffer() {
        let mut repl = Repl::new();
        for _ in 0..=INPUT_CAPACITY {
            repl.accept_printable(b'x');
        }

        assert_eq!(last_output(&repl), blank_line());
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

        assert_eq!(&last_output(&repl)[..17], b". -2147483648  ok");
    }
}
