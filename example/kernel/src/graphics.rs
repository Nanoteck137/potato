use boot_common::Framebuffer;
use spin::Mutex;

const FONT_BYTES: &'static [u8] = include_bytes!("../res/zap-vga16.psf");

enum PSFFontMode {
    NoUnicode256,
    NoUnicode512,
    Unicode256,
    Unicode512
}

struct PSFFont<'a> {
    bytes: &'a [u8],

    mode: PSFFontMode,
    char_size: u32,
}

impl<'a> PSFFont<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        if bytes[0] != 0x36 && bytes[1] != 0x04 {
            panic!("Invalid Magic for PSF Font version 1");
        }

        let mode = bytes[2];
        let mode = match mode {
            0 => PSFFontMode::NoUnicode256,
            1 => PSFFontMode::NoUnicode512,
            2 => PSFFontMode::Unicode256,
            3 => PSFFontMode::Unicode512,

            _ => panic!("Invalid PSF Font mode"),
        };

        let char_size = bytes[3] as u32;

        Self {
            bytes: &bytes[4..],
            mode,
            char_size
        }
    }

    fn put_char(&self, framebuffer: &Framebuffer, c: char, x: u32, y: u32) {
        let pixel_ptr = framebuffer.base as *mut u32;

        let mut offset = c as usize * self.char_size as usize;

        let x = x as isize;
        let y = y as isize;

        for yoff in 0..16 {
            for xoff in 0..8 {
                let data = self.bytes[offset] as u8;
                if (data & (0b10000000u8.wrapping_shr(xoff as u32))) > 0 {
                    unsafe {
                        let row_offset = (y + yoff) *
                            framebuffer.pixels_per_scanline as isize;
                        let pixel_offset =
                            (x + xoff) + row_offset;

                        core::ptr::write_volatile(pixel_ptr.offset(pixel_offset), 0xffffff);
                    }
                }
            }

            offset += 1;
        }
    }
}

struct Cursor {
    x: u32,
    y: u32,
}

struct Writer<'a> {
    font: PSFFont<'a>,
    framebuffer: &'a Framebuffer,

    cursor: Cursor,
}

impl<'a> Writer<'a> {
    fn new(font: PSFFont<'a>, framebuffer: &'a Framebuffer) -> Self {
        Self {
            font,
            framebuffer,

            cursor: Cursor {
                x: 0,
                y: 0
            }
        }
    }

    fn write_char(&mut self, c: char) {
        match c {
            '\n' => {
                self.cursor.x = 0;
                self.cursor.y += 1;
            }

            _ => {
                let x = self.cursor.x * 8;
                let y = self.cursor.y * 16;
                self.font.put_char(self.framebuffer, c, x, y);

                self.cursor.x += 1;
            }
        }
    }
}

impl<'a> core::fmt::Write for Writer<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }

        Ok(())
    }
}

static WRITER: Mutex<Option<Writer>> = Mutex::new(None);

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::graphics::_print(format_args!($($arg)*))
    );
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    WRITER.lock().as_mut().unwrap().write_fmt(args).unwrap();
}

pub fn init_graphics(framebuffer: &'static Framebuffer) {
    unsafe {
        core::ptr::write_bytes(framebuffer.base as *mut u8,
                               0, framebuffer.size as usize);
    }

    let font = PSFFont::new(FONT_BYTES);
    let writer = Writer::new(font, &framebuffer);

    {
        *WRITER.lock() = Some(writer);
    }
}
