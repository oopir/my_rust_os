use volatile::Volatile;         // for preventing optimizer to ignore VGA buffer
use core::fmt;                  // for string formatting features
use lazy_static::lazy_static;   // for having a static Writer without Rust issues
use spin::Mutex;                // for resolving issues with mutability & static vars

// -------------------------- enums & consts --------------------------

#[repr(u8)]             // save each value as u8 (meaning <256 value options)
#[allow(dead_code)]     // ignore the fact that we declared less than max (256)
#[derive(Debug, Clone, Copy, PartialEq, Eq)] 
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH:  usize = 80;

// -------------------------- helper structs --------------------------


#[repr(transparent)] // have the same representation as your base-type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}



#[repr(C)]  // make the struct's layout as it would be in C
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScreenChar {
    ascii_character: u8,
    color_code : ColorCode,
}



#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}


// --------------------------- writer struct ---------------------------


pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer  // will point to VGA buffer, so needs static lifetime
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n'  =>  self.new_line(),
            byte   =>  
            {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }
                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });

                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                // in the following line, 'read()' gets char from Volatile obj
                let ch = self.buffer.chars[row][col].read();  
                self.buffer.chars[row-1][col].write(ch);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        // create ScreenChar struct containing a space
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        // fill last line with spaces
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }

    }

    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n'  =>  self.write_byte(byte),
                _                    =>  self.write_byte(0xfe), 
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())  // this is a Result object, defined in rust core library
    }
}


// ----------------------- module's public WRITER -----------------------


lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}


// ---------------------- println macro definition ----------------------

// _print receives arguments in a format that fmt::Write knows to parse,
// and then uses our writer to print the given data
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;
    
    // disable interrupts to avoid deadlock from interrupts that print.
    // then, print.
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    })
}

// converts the given arguments to a format that the 'fmt' library knows,
// and then calls _print with the conversion's output
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

// same as 'print', but adds a newline character
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
