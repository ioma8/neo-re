pub const LCD_WIDTH: usize = 320;
pub const LCD_HEIGHT: usize = 128;

const CONTROLLER_WIDTH: usize = LCD_WIDTH / 2;
const RIGHT_CONTROLLER_X_BASE: usize = 132;
const PAGE_HEIGHT: usize = 8;
const PAGES: usize = LCD_HEIGHT / PAGE_HEIGHT;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LcdSnapshot {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<bool>,
}

#[derive(Clone, Debug)]
pub(crate) struct Lcd {
    controllers: [Controller; 2],
}

#[derive(Clone, Copy, Debug)]
struct Controller {
    page: usize,
    column: usize,
    display_start_line: usize,
    read_modify_write: Option<usize>,
    display_on: bool,
    reverse_display: bool,
    pixels: [[u8; CONTROLLER_WIDTH]; PAGES],
}

impl Lcd {
    pub(crate) fn new() -> Self {
        Self {
            controllers: [Controller::new(), Controller::new()],
        }
    }

    pub(crate) fn write_command(&mut self, controller: usize, command: u8) {
        if let Some(controller) = self.controllers.get_mut(controller) {
            controller.write_command(command);
        }
    }

    pub(crate) fn write_data(&mut self, controller: usize, value: u8) {
        if let Some(controller) = self.controllers.get_mut(controller) {
            controller.write_data(value);
        }
    }

    pub(crate) fn read_status(&self, controller: usize) -> u8 {
        self.controllers
            .get(controller)
            .map_or(0, Controller::read_status)
    }

    pub(crate) fn read_data(&mut self, controller: usize) -> u8 {
        self.controllers
            .get_mut(controller)
            .map_or(0, Controller::read_data)
    }

    pub(crate) fn snapshot(&self) -> LcdSnapshot {
        let mut pixels = vec![false; LCD_WIDTH * LCD_HEIGHT];
        for controller_index in 0..self.controllers.len() {
            let controller = self.controllers[controller_index];
            let x_base = controller_x_base(controller_index);
            for page in 0..PAGES {
                for column in 0..CONTROLLER_WIDTH {
                    let value = controller.pixels[page][column];
                    for bit in 0..PAGE_HEIGHT {
                        let x = x_base + column;
                        let y = page * PAGE_HEIGHT + bit;
                        if x < LCD_WIDTH {
                            let lit = value & (1 << bit) != 0;
                            pixels[y * LCD_WIDTH + x] =
                                controller.display_on && (lit ^ controller.reverse_display);
                        }
                    }
                }
            }
        }
        LcdSnapshot {
            width: LCD_WIDTH,
            height: LCD_HEIGHT,
            pixels,
        }
    }
}

fn controller_x_base(controller_index: usize) -> usize {
    if controller_index == 0 {
        0
    } else {
        RIGHT_CONTROLLER_X_BASE
    }
}

impl Controller {
    const fn new() -> Self {
        Self {
            page: 0,
            column: 0,
            display_start_line: 0,
            read_modify_write: None,
            display_on: true,
            reverse_display: false,
            pixels: [[0; CONTROLLER_WIDTH]; PAGES],
        }
    }

    fn write_command(&mut self, command: u8) {
        match command {
            0xae => self.display_on = false,
            0xaf => self.display_on = true,
            0xb0..=0xbf => self.page = usize::from(command & 0x0f).min(PAGES - 1),
            0x00..=0x0f => self.column = (self.column & 0xf0) | usize::from(command),
            0x10..=0x1f => self.column = (self.column & 0x0f) | (usize::from(command & 0x0f) << 4),
            0x40..=0x7f => self.display_start_line = usize::from(command & 0x3f),
            0xa6 => self.reverse_display = false,
            0xa7 => self.reverse_display = true,
            0xe0 => self.read_modify_write = Some(self.column),
            0xee => {
                if let Some(column) = self.read_modify_write.take() {
                    self.column = column;
                }
            }
            0xa0 | 0xa1 | 0xa3 | 0xc0..=0xc8 | 0xf8 => {}
            _ => {}
        }
        self.column = self.column.min(CONTROLLER_WIDTH - 1);
    }

    fn write_data(&mut self, value: u8) {
        self.pixels[self.page][self.column] = value;
        self.column = (self.column + 1).min(CONTROLLER_WIDTH - 1);
    }

    fn read_status(&self) -> u8 {
        if self.display_on { 0x00 } else { 0x20 }
    }

    fn read_data(&mut self) -> u8 {
        let value = self.pixels[self.page][self.column];
        if self.read_modify_write.is_none() {
            self.column = (self.column + 1).min(CONTROLLER_WIDTH - 1);
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use super::{LCD_WIDTH, Lcd};

    #[test]
    fn data_write_sets_vertical_pixels_and_advances_column() {
        let mut lcd = Lcd::new();

        lcd.write_command(0, 0xb1);
        lcd.write_command(0, 0x11);
        lcd.write_command(0, 0x02);
        lcd.write_data(0, 0b0000_0101);

        let snapshot = lcd.snapshot();
        let x = 0x12;
        assert!(snapshot.pixels[8 * LCD_WIDTH + x]);
        assert!(!snapshot.pixels[9 * LCD_WIDTH + x]);
        assert!(snapshot.pixels[10 * LCD_WIDTH + x]);
    }

    #[test]
    fn right_controller_maps_to_right_half() {
        let mut lcd = Lcd::new();

        lcd.write_command(1, 0xb0);
        lcd.write_data(1, 0x01);

        let snapshot = lcd.snapshot();
        assert!(snapshot.pixels[132]);
        assert!(!snapshot.pixels[0]);
    }

    #[test]
    fn reverse_display_command_inverts_visible_pixels() {
        let mut lcd = Lcd::new();

        lcd.write_command(0, 0xa7);
        lcd.write_command(0, 0xb0);
        lcd.write_data(0, 0x00);

        let snapshot = lcd.snapshot();
        assert!(snapshot.pixels[0]);
    }

    #[test]
    fn display_start_line_command_does_not_reset_column() {
        let mut lcd = Lcd::new();

        lcd.write_command(0, 0xb0);
        lcd.write_command(0, 0x10);
        lcd.write_command(0, 0x05);
        lcd.write_command(0, 0x40);
        lcd.write_data(0, 0x01);

        let snapshot = lcd.snapshot();
        assert!(!snapshot.pixels[0]);
        assert!(snapshot.pixels[5]);
    }

    #[test]
    fn data_read_returns_framebuffer_byte_and_advances_column() {
        let mut lcd = Lcd::new();

        lcd.write_command(0, 0xb0);
        lcd.write_command(0, 0x03);
        lcd.write_data(0, 0x5a);
        lcd.write_command(0, 0x03);

        assert_eq!(lcd.read_data(0), 0x5a);
        lcd.write_data(0, 0x33);

        let snapshot = lcd.snapshot();
        assert!(snapshot.pixels[LCD_WIDTH + 3]);
        assert!(snapshot.pixels[4]);
    }

    #[test]
    fn read_modify_write_read_does_not_advance_column_until_write() {
        let mut lcd = Lcd::new();

        lcd.write_command(0, 0xb0);
        lcd.write_command(0, 0x04);
        lcd.write_data(0, 0x5a);
        lcd.write_command(0, 0x04);
        lcd.write_command(0, 0xe0);

        assert_eq!(lcd.read_data(0), 0x5a);
        lcd.write_data(0, 0x00);

        let snapshot = lcd.snapshot();
        assert!(!snapshot.pixels[4]);
    }
}
