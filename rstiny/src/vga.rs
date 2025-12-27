// 从启动日志获取的帧缓冲信息
const FB_ADDR: usize = 0xecd20000;
const FB_WIDTH: usize = 1920;
const FB_HEIGHT: usize = 1200;
const FB_STRIDE: usize = 7680;  // 每行字节数
const BYTES_PER_PIXEL: usize = 4; // 32位 = 4字节

use core::ptr::NonNull;

use font8x8::{UnicodeFonts, BASIC_FONTS};

// 颜色定义 (根据 Linux 日志: shift=24:16:8:0，格式为 0xAARRGGBB)
// Alpha在最高字节(24-31位), Red(16-23位), Green(8-15位), Blue(0-7位)
const COLOR_BLACK: u32 = 0x00000000;
const COLOR_WHITE: u32 = 0x00FFFFFF;
const COLOR_RED: u32 = 0x00FF0000;
const COLOR_GREEN: u32 = 0x0000FF00;
const COLOR_BLUE: u32 = 0x000000FF;
const COLOR_YELLOW: u32 = 0x00FFFF00;
const COLOR_CYAN: u32 = 0x0000FFFF;
const COLOR_MAGENTA: u32 = 0x00FF00FF;

pub struct FrameBuffer {
    base: *mut u32,
    width:  usize,
    height: usize,
    stride_pixels: usize, // 每行像素数（stride / 4）
    cursor_x: usize,  // 文本光标 X 位置（像素）
    cursor_y: usize,  // 文本光标 Y 位置（像素）
    char_width: usize,   // 字符宽度（像素）
    char_height: usize,  // 字符高度（像素）
    char_scale: usize,   // 字符放大倍数
}

// SAFETY: FrameBuffer 只包含 MMIO 内存地址和基础类型，可以在线程间安全传递
// 实际的访问由 SpinNoIrq 互斥锁保护
unsafe impl Send for FrameBuffer {}
unsafe impl Sync for FrameBuffer {}

impl FrameBuffer {
    pub fn new(base_addr: NonNull<usize>) -> Self {
        Self {
            base: base_addr.as_ptr() as *mut u32,
            width: FB_WIDTH,
            height: FB_HEIGHT,
            stride_pixels: FB_STRIDE / BYTES_PER_PIXEL,
            cursor_x: 0,
            cursor_y: 0,
            char_width: 8,
            char_height: 8,
            char_scale: 2,  // 默认放大 2 倍
        }
    }

    // 画单个像素
    pub fn draw_pixel(&mut self, x: usize, y: usize, color: u32) {
        if x < self.width && y < self.height {
            unsafe {
                let offset = y * self.stride_pixels + x;
                core::ptr::write_volatile(self.base.add(offset), color);
            }
        }
    }

    // 填充矩形
    pub fn fill_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: u32) {
        for dy in 0..height {
            for dx in 0..width {
                self.draw_pixel(x + dx, y + dy, color);
            }
        }
    }

    // 清屏
    pub fn clear(&mut self, color: u32) {
        self.fill_rect(0, 0, self.width, self.height, color);
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    // 绘制单个字符
    pub fn draw_char(&mut self, ch: char, x: usize, y: usize, fg_color: u32, bg_color: u32) {
        let glyph = ascii_to_matrix(ch);
        let _scaled_width = self.char_width * self.char_scale;
        let _scaled_height = self.char_height * self.char_scale;
        
        for (row, byte) in glyph.iter().enumerate() {
            for col in 0..8 {
                let is_set = (byte & (1 << col)) != 0;
                let color = if is_set { fg_color } else { bg_color };
                
                // 绘制放大的像素块
                for dy in 0..self.char_scale {
                    for dx in 0..self.char_scale {
                        self.draw_pixel(
                            x + col * self.char_scale + dx,
                            y + row * self.char_scale + dy,
                            color
                        );
                    }
                }
            }
        }
    }

    // 向屏幕滚动一行
    fn scroll_up(&mut self) {
        let line_height = self.char_height * self.char_scale;
        let scroll_bytes = (self.height - line_height) * self.stride_pixels;
        
        unsafe {
            // 将除了最后一行外的所有内容向上移动一行
            core::ptr::copy(
                self.base.add(line_height * self.stride_pixels),
                self.base,
                scroll_bytes
            );
            
            // 清空最后一行
            let last_line_start = (self.height - line_height) * self.stride_pixels;
            for i in 0..(line_height * self.stride_pixels) {
                core::ptr::write_volatile(
                    self.base.add(last_line_start + i),
                    COLOR_BLACK
                );
            }
        }
    }

    // 写入一个字符（处理换行和滚动）
    pub fn write_char(&mut self, ch: char) {
        let scaled_width = self.char_width * self.char_scale;
        let scaled_height = self.char_height * self.char_scale;
        let margin = 4; // 字符间距
        
        match ch {
            '\n' => {
                self.cursor_x = 0;
                self.cursor_y += scaled_height;
            }
            '\r' => {
                self.cursor_x = 0;
            }
            ch => {
                // 检查是否需要换行
                if self.cursor_x + scaled_width > self.width {
                    self.cursor_x = 0;
                    self.cursor_y += scaled_height;
                }
                
                // 检查是否需要滚动
                if self.cursor_y + scaled_height > self.height {
                    self.scroll_up();
                    self.cursor_y = self.height - scaled_height;
                }
                
                // 绘制字符
                self.draw_char(ch, self.cursor_x, self.cursor_y, COLOR_WHITE, COLOR_BLACK);
                self.cursor_x += scaled_width + margin;
            }
        }
    }

    // 写入字符串
    pub fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }
}

// 将 ASCII 字符转换为 8x8 的位图矩阵
// 使用 font8x8 crate 提供完整的 ASCII 字符集支持
pub fn ascii_to_matrix(ch: char) -> [u8; 8] {
    BASIC_FONTS
        .get(ch)
        .unwrap_or(BASIC_FONTS.get('?').unwrap())
}

// 全局静态 FrameBuffer（用于实现 print 宏）
use kspin::SpinNoIrq;
use lazyinit::LazyInit;

static FRAMEBUFFER: LazyInit<SpinNoIrq<FrameBuffer>> = LazyInit::new();

/// 初始化全局 Framebuffer（在 main 函数中调用一次）
pub fn init() {
    use core::ptr::NonNull;
    
    const FB_PADDR: usize = 0xecd20000;
    let base_addr = NonNull::new(FB_PADDR as *mut usize).expect("Invalid framebuffer address");
    let fb = FrameBuffer::new(base_addr);
    FRAMEBUFFER.init_once(SpinNoIrq::new(fb));
}

// 实现 core::fmt::Write trait 以支持 write! 宏
impl core::fmt::Write for FrameBuffer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

// print 宏辅助函数
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    FRAMEBUFFER.lock().write_fmt(args).unwrap();
}

// print! 宏
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

// println! 宏
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn show_text() -> ! {
    // 清屏
    FRAMEBUFFER.lock().clear(COLOR_BLACK);
    
    // 使用新的文本输出功能
    {
        let mut fb = FRAMEBUFFER.lock();
        fb.write_str("Welcome to ArceOS!\n");
        fb.write_str("Font8x8 ASCII Display Test\n\n");
        fb.write_str("ASCII: 0123456789\n");
        fb.write_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ\n");
        fb.write_str("abcdefghijklmnopqrstuvwxyz\n\n");
        fb.write_str("System initialized successfully!\n");
    }
    
    // 演示 print! 和 println! 宏
    println!();
    println!("=== Print Macro Test ===");
    print!("This is print! without newline. ");
    println!("This is println!");
    println!("Number: {}, Hex: 0x{:x}", 42, 255);
    
    loop {
        core::hint::spin_loop();
    }
}

pub fn show_img() -> ! {
   const FB_PADDR: usize = 0xffff0000ecd20000;
    let base_addr = NonNull::new(FB_PADDR as *mut usize).expect("Invalid framebuffer address");
    let mut fb = FrameBuffer::new(base_addr);
    
    // 清屏为黑色
    fb.clear(COLOR_BLACK);

    fb.write_str("Displaying text using font8x8!\n");
    
    // 测试 font8x8 字体显示
    let test_text = "Hello ArceOS!";
    let start_x = 100;
    let start_y = 100;
    let scale = 6; // 放大倍数
    let char_spacing = 8 * scale + 10; // 字符间距
    
    for (i, ch) in test_text.chars().enumerate() {
        let glyph = ascii_to_matrix(ch);
        let char_x = start_x + i * char_spacing;
        
        // font8x8 返回 [u8; 8]，每个字节代表一行的 8 个像素
        for (row, byte) in glyph.iter().enumerate() {
            for col in 0..8 {
                // 检查该位是否设置（从最低位开始）
                let is_set = (byte & (1 << col)) != 0;
                let color = if is_set { COLOR_WHITE } else { COLOR_BLACK };
                
                // 绘制放大的像素块
                for dy in 0..scale {
                    for dx in 0..scale {
                        fb.draw_pixel(
                            char_x + col * scale + dx,
                            start_y + row * scale + dy,
                            color
                        );
                    }
                }
            }
        }
    }
    
    // 显示更多文本示例 - 显示 ASCII 表
    let start_y2 = 300;
    let demo_text = "0123456789 ABCDEFG";
    
    for (i, ch) in demo_text.chars().enumerate() {
        let glyph = ascii_to_matrix(ch);
        let char_x = start_x + i * char_spacing;
        
        for (row, byte) in glyph.iter().enumerate() {
            for col in 0..8 {
                let is_set = (byte & (1 << col)) != 0;
                let color = if is_set { COLOR_GREEN } else { COLOR_BLACK };
                
                for dy in 0..scale {
                    for dx in 0..scale {
                        fb.draw_pixel(
                            char_x + col * scale + dx,
                            start_y2 + row * scale + dy,
                            color
                        );
                    }
                }
            }
        }
    }
    
    loop {
        core::hint::spin_loop();
    }
}