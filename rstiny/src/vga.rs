// 从启动日志获取的帧缓冲信息
const FB_ADDR: usize = 0xecd20000;
const FB_WIDTH: usize = 1920;
const FB_HEIGHT: usize = 1200;
const FB_STRIDE: usize = 7680;  // 每行字节数
const BYTES_PER_PIXEL: usize = 4; // 32位 = 4字节

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    text::Text,
    Drawable, Pixel,
};

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
}

impl FrameBuffer {
    pub fn new() -> Self {
        Self {
            base: FB_ADDR as *mut u32,
            width: FB_WIDTH,
            height: FB_HEIGHT,
            stride_pixels: FB_STRIDE / BYTES_PER_PIXEL,
        }
    }

    // 将 Rgb888 转换为 u32 颜色值
    fn rgb888_to_u32(color: Rgb888) -> u32 {
        let r = color.r() as u32;
        let g = color.g() as u32;
        let b = color.b() as u32;
        (r << 16) | (g << 8) | b
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
    }

    // 画空心圆（使用中点画圆算法）
    pub fn draw_circle(&mut self, cx: usize, cy: usize, radius: usize, color: u32) {
        let mut x = radius as i32;
        let mut y = 0i32;
        let mut err = 0i32;

        while x >= y {
            self.plot_circle_points(cx, cy, x, y, color);
            
            if err <= 0 {
                y += 1;
                err += 2 * y + 1;
            }
            if err > 0 {
                x -= 1;
                err -= 2 * x + 1;
            }
        }
    }

    fn plot_circle_points(&mut self, cx: usize, cy: usize, x: i32, y: i32, color: u32) {
        let points = [
            (cx as i32 + x, cy as i32 + y),
            (cx as i32 - x, cy as i32 + y),
            (cx as i32 + x, cy as i32 - y),
            (cx as i32 - x, cy as i32 - y),
            (cx as i32 + y, cy as i32 + x),
            (cx as i32 - y, cy as i32 + x),
            (cx as i32 + y, cy as i32 - x),
            (cx as i32 - y, cy as i32 - x),
        ];
        
        for (px, py) in points {
            if px >= 0 && py >= 0 {
                self.draw_pixel(px as usize, py as usize, color);
            }
        }
    }

    // 画实心圆
    pub fn fill_circle(&mut self, cx: usize, cy: usize, radius: usize, color: u32) {
        let r2 = (radius * radius) as i32;
        for dy in -(radius as i32)..=(radius as i32) {
            for dx in -(radius as i32)..=(radius as i32) {
                if dx * dx + dy * dy <= r2 {
                    let px = cx as i32 + dx;
                    let py = cy as i32 + dy;
                    if px >= 0 && py >= 0 {
                        self.draw_pixel(px as usize, py as usize, color);
                    }
                }
            }
        }
    }

    // 画三角形（空心）
    pub fn draw_triangle(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, x3: usize, y3: usize, color: u32) {
        self.draw_line(x1, y1, x2, y2, color);
        self.draw_line(x2, y2, x3, y3, color);
        self.draw_line(x3, y3, x1, y1, color);
    }

    // 画线（Bresenham算法）
    pub fn draw_line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
        let mut x0 = x0 as i32;
        let mut y0 = y0 as i32;
        let x1 = x1 as i32;
        let y1 = y1 as i32;
        
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        
        loop {
            if x0 >= 0 && y0 >= 0 {
                self.draw_pixel(x0 as usize, y0 as usize, color);
            }
            
            if x0 == x1 && y0 == y1 { break; }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    // 画数字0（使用简单的7段显示风格）
    pub fn draw_digit_0(&mut self, x: usize, y: usize, size: usize, color: u32) {
        // 画一个圆形的0
        let radius = size / 2;
        let cx = x + radius;
        let cy = y + radius;
        
        // 画外圆和内圆，形成环形
        for r in 0..=3 {
            self.draw_circle(cx, cy, radius + r, color);
        }
    }
}

// 实现 embedded-graphics 的 DrawTarget trait
impl DrawTarget for FrameBuffer {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(coord, color) in pixels.into_iter() {
            let x = coord.x as usize;
            let y = coord.y as usize;
            if x < self.width && y < self.height {
                let color_u32 = Self::rgb888_to_u32(color);
                unsafe {
                    let offset = y * self.stride_pixels + x;
                    core::ptr::write_volatile(self.base.add(offset), color_u32);
                }
            }
        }
        Ok(())
    }
}

// 实现 OriginDimensions trait
impl OriginDimensions for FrameBuffer {
    fn size(&self) -> Size {
        Size::new(self.width as u32, self.height as u32)
    }
}

pub fn show_img() -> ! {
    let mut fb = FrameBuffer::new();
    
    // 清屏为黑色
    fb.clear(COLOR_BLACK);
    
    // 只画几个简单的方块测试
    fb.fill_rect(100, 100, 200, 150, COLOR_RED);
    fb.fill_rect(350, 100, 200, 150, COLOR_GREEN);
    fb.fill_rect(600, 100, 200, 150, COLOR_BLUE);
    
    // 画一个白色方块
    fb.fill_rect(100, 300, 200, 150, COLOR_WHITE);
    
    // 停在这里
    loop {
        core::hint::spin_loop();
    }
}