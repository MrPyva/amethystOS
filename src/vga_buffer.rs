use core::fmt;
use core::fmt::{Arguments, Write};
use core::ops::DerefMut;
use core::ptr::write;
use x86_64::instructions::interrupts::without_interrupts;
use x86_64::instructions::port::Port;
use crate::spin_mutex::SpinlockMutexGuard;
use crate::{SpinlockMutex, VGA_BUFFER};

const VGA_BUFFER_PTR: *mut u16 = 0xb8000 as *mut u16;
const CHAR_MAX: usize = 80;
const LINE_MAX: usize = 25;

static mut VGA_WRITER: Option<SpinlockMutex<VGAWriter>> = None;

pub fn load() {
    unsafe {
        if VGA_WRITER.is_some() {
            return;
        }

        VGA_WRITER = Some(SpinlockMutex::new(VGAWriter::new()));
    }
}

fn get<'a>() -> SpinlockMutexGuard<'a, VGAWriter> {
    unsafe {VGA_WRITER.as_ref().unwrap().spinlock()}
}

pub enum Color {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    Pink,
    Yellow,
    White
}

pub struct ColorText {
    pub value: u8
}

impl ColorText {
    pub fn new(foreground_color: Color, background_color: Color) -> Self {
        ColorText {value: (background_color as u8) << 4 | (foreground_color as u8)}
    }
}

impl Default for ColorText {
    fn default() -> Self {
        ColorText::new(Color::White, Color::Black)
    }
}

pub struct VGAWriter {
    pub line_counter: usize,
    pub char_counter: usize,
    buffer: &'static mut Buffer
}

#[repr(C)]
struct Buffer {
    data: [[u16; CHAR_MAX]; LINE_MAX]
}

impl VGAWriter {
    pub fn new() -> Self {
        unsafe {VGAWriter {char_counter: 0, line_counter : 0, buffer: &mut *(VGA_BUFFER_PTR as *mut Buffer)}}
    }

    pub fn update_cursor(&mut self) {
        let mut x3d4 = Port::<u8>::new(0x3D4);
        let mut x3d5 = Port::<u8>::new(0x3D5);

        let pos = (self.line_counter * 80 + self.char_counter) as u16;

        if self.buffer.data[self.line_counter][self.char_counter] == 0 {
            self.buffer.data[self.line_counter][self.char_counter] = (ColorText::default().value as u16) << 8
        }

        unsafe {
            x3d4.write(0x0F);
            x3d5.write((pos & 0xFF) as u8);
            x3d4.write(0x0E);
            x3d5.write(((pos >> 8) & 0xFF) as u8);
        }
    }

    pub fn backspace(&mut self) {
        if self.char_counter == 0 {
            self.line_counter = self.line_counter.saturating_sub(1);
            self.char_counter = 79;
            self.buffer.data[self.line_counter][self.char_counter] = 0;
            self.update_cursor();
            return;
        }

        self.char_counter = self.char_counter.saturating_sub(1);
        self.buffer.data[self.line_counter][self.char_counter] = 0;
        self.update_cursor();
    }

    pub fn write_bytes(&mut self, msg: &[u8], color: ColorText) {
        for &x in msg {
            match x {
                b'\n' => {
                    self.new_line();
                    continue;
                }
                0x20..=0x7e => self.buffer.data[self.line_counter][self.char_counter] = ((color.value as u16) << 8) | x as u16,
                _ => self.buffer.data[self.line_counter][self.char_counter] = ((color.value as u16) << 8) | 0 as u16,
            }

            self.char_counter += 1;
            if self.char_counter >= CHAR_MAX {
                self.new_line();
            }
        }

        self.update_cursor();
    }

    pub fn new_line(&mut self) {
        self.char_counter = 0;
        self.line_counter += 1;

        if self.line_counter >= LINE_MAX {
            self.shift_up();
            self.line_counter -= 1;
        }
    }

    pub fn shift_up(&mut self) {
        for x in 1..LINE_MAX {
            for y in 0..CHAR_MAX {
                self.buffer.data[x - 1][y] = self.buffer.data[x][y]
            }
        }

        for y in 0..CHAR_MAX {
            self.buffer.data[LINE_MAX - 1][y] = 0
        }
    }

    pub fn write(&mut self, msg: &str, color: ColorText) {
        self.write_bytes(msg.as_bytes(), color);
    }

    pub fn log(&mut self, msg: &str, prefix: &str, prefix_color: ColorText) {
        if self.char_counter > 0 {
            self.write("\n", ColorText::default());
        }
        self.write(prefix, prefix_color);
        self.write(msg, ColorText::default());
    }


    pub fn info(&mut self, msg: &str) {
        self.log(msg, "[INFO] ", ColorText::new(Color::LightBlue, Color::Black));
    }

    pub fn warn(&mut self, msg: &str) {
        self.log(msg, "[WARNING] ", ColorText::new(Color::Yellow, Color::Black));
    }

    pub fn error(&mut self, msg: &str) {
        self.log(msg, "[ERROR] ", ColorText::new(Color::LightRed, Color::Black));
    }

    pub fn fatal(&mut self, msg: &str) {
        self.log(msg, "[FATAL] ", ColorText::new(Color::Red, Color::Black));
    }

    pub fn debug(&mut self, msg: &str) {
        self.log(msg, "[DEBUG] ", ColorText::default());
    }
}

pub fn backspace() {
    without_interrupts(|| {
        get().backspace();
    });
}

pub fn write_fmt(args: Arguments<'_>) {
    without_interrupts(|| {
        get().write_fmt(args);
    });
}

pub fn write_str(s: &str) {
    without_interrupts(|| {
        get().write_str(s);
    });
}

pub fn log(msg: &str, prefix: &str, prefix_color: ColorText) {
    without_interrupts(|| {
        get().log(msg, prefix, prefix_color);
    });
}

pub fn info(msg: &str) {
    log(msg, "[INFO] ", ColorText::new(Color::LightBlue, Color::Black));
}

pub fn warn(msg: &str) {
    log(msg, "[WARNING] ", ColorText::new(Color::Yellow, Color::Black));
}

pub fn error(msg: &str) {
    log(msg, "[ERROR] ", ColorText::new(Color::LightRed, Color::Black));
}

pub fn fatal(msg: &str) {
    log(msg, "[FATAL] ", ColorText::new(Color::Red, Color::Black));
}

pub fn debug(msg: &str) {
    log(msg, "[DEBUG] ", ColorText::default());
}

impl Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s, ColorText::default());
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! debugn {
    ($($arg:tt)*) => ($crate::vga_buffer::_debug(format_args!($($arg)*)));
}
#[macro_export]
macro_rules! debug {
    () => ($crate::debugn!("\n"));
    ($($arg:tt)*) => ($crate::debugn!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    get().write_fmt(args);
}

#[doc(hidden)]
pub fn _debug(args: fmt::Arguments) {
    use core::fmt::Write;

    let mut vga_buffer = get();
    if vga_buffer.char_counter > 0 {
        vga_buffer.write("\n", ColorText::default());
    }
    vga_buffer.write("[DEBUG]", ColorText::default());
    vga_buffer.write_fmt(args);
}