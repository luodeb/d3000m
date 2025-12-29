use core::sync::atomic::AtomicUsize;

use font8x8::{UnicodeFonts, BASIC_FONTS};
static CURRENT_Y: AtomicUsize = AtomicUsize::new(0);

pub fn ascii_to_matrix(ch: char) -> [u8; 8] {
    BASIC_FONTS
        .get(ch)
        .unwrap_or(BASIC_FONTS.get('?').unwrap())
}

pub fn draw_pixel(x: usize, y: usize, color: u32) {
    unsafe {
        let offset = y * 1920 + x;
        core::ptr::write_volatile((0xffff_0000_ecd2_0000 as *mut u32).add(offset), color);
    }
}

pub fn draw_char(ch: char, x: usize, y: usize, fg_color: u32, bg_color: u32) {
    let glyph = ascii_to_matrix(ch);

    for (row, byte) in glyph.iter().enumerate() {
        for col in 0..8 {
            let is_set = (byte & (1 << col)) != 0;
            let color = if is_set { fg_color } else { bg_color };

            // 绘制放大的像素块
            for dy in 0..1 {
                for dx in 0..1 {
                    draw_pixel(
                        x + col * 1 + dx,
                        y + row * 1 + dy,
                        color,
                    );
                }
            }
        }
    }
}

pub fn draw_string(s: &str) {
    let y = CURRENT_Y.load(core::sync::atomic::Ordering::SeqCst);
    for (i, ch) in s.chars().enumerate() {
        match ch {
            '\n' => {
                CURRENT_Y.fetch_add(10, core::sync::atomic::Ordering::SeqCst);
                if CURRENT_Y.load(core::sync::atomic::Ordering::SeqCst) >= 1200 {
                    CURRENT_Y.store(0, core::sync::atomic::Ordering::SeqCst);
                }
                continue;
            }
            _ => {}
        }
        draw_char(ch, 10 + i * 8, y, 0xFFFFFF, 0x000000);
    }
}

pub fn draw_args(s: &core::fmt::Arguments) {
    use core::fmt::Write;
    struct VGAWriter;

    impl core::fmt::Write for VGAWriter {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            draw_string(s);
            Ok(())
        }
    }

    let mut writer = VGAWriter;
    writer.write_fmt(*s).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::vga::draw_args(&format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}
