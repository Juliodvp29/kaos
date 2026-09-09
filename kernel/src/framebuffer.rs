//! Pixel framebuffer text output.
//!
//! Renders text over the raw framebuffer handed over by the bootloader, using
//! the pre-rasterized Noto Sans Mono glyphs. All rendering math is pure (it
//! only touches the borrowed pixel slice), so it is covered by host unit
//! tests; see the `tests` module at the bottom. The global writer installed
//! by [`init`] is the only shared state, guarded by a `spin::Mutex` because
//! later phases will print from interrupt handlers.

use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use core::fmt;
use noto_sans_mono_bitmap::{
    FontWeight, RasterHeight, RasterizedChar, get_raster, get_raster_width,
};
use spin::Mutex;

/// RGB color with 8 bits per channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    /// Screen background in Phase 1.
    pub const BLACK: Self = Self {
        red: 0,
        green: 0,
        blue: 0,
    };
    /// Default glyph color in Phase 1.
    pub const WHITE: Self = Self {
        red: 255,
        green: 255,
        blue: 255,
    };
}

const FONT_WEIGHT: FontWeight = FontWeight::Regular;
const FONT_HEIGHT: RasterHeight = RasterHeight::Size16;

/// Width of one glyph cell in pixels.
fn glyph_width() -> usize {
    get_raster_width(FONT_WEIGHT, FONT_HEIGHT)
}

/// Height of one glyph cell in pixels.
fn glyph_height() -> usize {
    FONT_HEIGHT.val()
}

/// Blends `foreground` over `background` with the given glyph intensity
/// (0 = transparent, 255 = fully opaque).
fn blend(background: Color, foreground: Color, intensity: u8) -> Color {
    let intensity = u16::from(intensity);
    let mix = |background: u8, foreground: u8| {
        let background = u16::from(background);
        let foreground = u16::from(foreground);
        ((background * (255 - intensity) + foreground * intensity) / 255) as u8
    };
    Color {
        red: mix(background.red, foreground.red),
        green: mix(background.green, foreground.green),
        blue: mix(background.blue, foreground.blue),
    }
}

/// Text-mode writer over a bootloader framebuffer.
pub struct FrameBufferWriter {
    buffer: &'static mut [u8],
    info: FrameBufferInfo,
    column: usize,
    row: usize,
    foreground: Color,
    background: Color,
}

impl FrameBufferWriter {
    /// Creates a writer over the given framebuffer and clears the screen.
    ///
    /// The buffer must match `info` (same byte length the bootloader
    /// reported); a mismatch only triggers a debug assertion because the
    /// bootloader is trusted to describe its own framebuffer correctly.
    pub fn new(buffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        debug_assert_eq!(buffer.len(), info.byte_len);
        let mut writer = Self {
            buffer,
            info,
            column: 0,
            row: 0,
            foreground: Color::WHITE,
            background: Color::BLACK,
        };
        writer.clear();
        writer
    }

    /// Number of text columns that fit on screen.
    fn columns(&self) -> usize {
        self.info.width / glyph_width()
    }

    /// Number of text rows that fit on screen.
    fn rows(&self) -> usize {
        self.info.height / glyph_height()
    }

    /// Clears the screen to the background color and resets the cursor.
    pub fn clear(&mut self) {
        let background = self.background;
        let (width, height) = (self.info.width, self.info.height);
        for y in 0..height {
            for x in 0..width {
                self.write_pixel(x, y, background);
            }
        }
        self.column = 0;
        self.row = 0;
    }

    /// Advances to the next line, scrolling the screen when full.
    fn newline(&mut self) {
        self.column = 0;
        self.row += 1;
        if self.row >= self.rows() {
            self.scroll_up();
            self.row = self.rows().saturating_sub(1);
        }
    }

    /// Shifts the visible text up by one glyph row and clears the last row.
    fn scroll_up(&mut self) {
        let line_bytes = glyph_height() * self.info.stride * self.info.bytes_per_pixel;
        if line_bytes < self.buffer.len() {
            self.buffer.copy_within(line_bytes.., 0);
        }
        let background = self.background;
        let clear_top = self.info.height.saturating_sub(glyph_height());
        for y in clear_top..self.info.height {
            for x in 0..self.info.width {
                self.write_pixel(x, y, background);
            }
        }
    }

    /// Paints one pixel, silently ignoring out-of-bounds coordinates and
    /// unknown pixel formats instead of panicking.
    fn write_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.info.width || y >= self.info.height {
            return;
        }
        let bytes_per_pixel = self.info.bytes_per_pixel;
        if bytes_per_pixel == 0 || bytes_per_pixel > 4 {
            return;
        }
        let byte_offset = (y * self.info.stride + x) * bytes_per_pixel;
        if byte_offset + bytes_per_pixel > self.buffer.len() {
            return;
        }
        let bytes: [u8; 4] = match self.info.pixel_format {
            PixelFormat::Rgb => [color.red, color.green, color.blue, 0],
            PixelFormat::Bgr => [color.blue, color.green, color.red, 0],
            PixelFormat::U8 => {
                let gray = ((u16::from(color.red) + u16::from(color.green) + u16::from(color.blue))
                    / 3) as u8;
                [gray, 0, 0, 0]
            }
            // Unknown firmware pixel layout: skip instead of corrupting the screen.
            _ => return,
        };
        self.buffer[byte_offset..byte_offset + bytes_per_pixel]
            .copy_from_slice(&bytes[..bytes_per_pixel]);
    }

    /// Renders one glyph at the cursor position and advances the cursor.
    fn write_glyph(&mut self, glyph: RasterizedChar) {
        let (foreground, background) = (self.foreground, self.background);
        for (glyph_y, glyph_row) in glyph.raster().iter().enumerate() {
            for (glyph_x, intensity) in glyph_row.iter().enumerate() {
                let x = self.column * glyph_width() + glyph_x;
                let y = self.row * glyph_height() + glyph_y;
                self.write_pixel(x, y, blend(background, foreground, *intensity));
            }
        }
        self.column += 1;
        if self.column >= self.columns() {
            self.newline();
        }
    }

    /// Writes one character, handling control characters. Characters missing
    /// from the font fall back to `?`, and are skipped if even that is absent.
    fn write_char(&mut self, character: char) {
        if self.columns() == 0 || self.rows() == 0 {
            return;
        }
        match character {
            '\n' => self.newline(),
            '\r' => self.column = 0,
            character => {
                let glyph = get_raster(character, FONT_WEIGHT, FONT_HEIGHT)
                    .or_else(|| get_raster('?', FONT_WEIGHT, FONT_HEIGHT));
                if let Some(glyph) = glyph {
                    self.write_glyph(glyph);
                }
            }
        }
    }
}

impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for character in text.chars() {
            self.write_char(character);
        }
        Ok(())
    }
}

/// Global writer installed by [`init`]. `None` until the kernel entry point
/// takes ownership of the bootloader framebuffer.
static WRITER: Mutex<Option<FrameBufferWriter>> = Mutex::new(None);

/// Installs the global writer over the bootloader framebuffer.
pub fn init(buffer: &'static mut [u8], info: FrameBufferInfo) {
    *WRITER.lock() = Some(FrameBufferWriter::new(buffer, info));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    if let Some(writer) = WRITER.lock().as_mut() {
        // Screen output is best-effort; it must never panic.
        let _ = writer.write_fmt(args);
    }
}

/// Prints to the framebuffer without a trailing newline.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::framebuffer::_print(core::format_args!($($arg)*)));
}

/// Prints to the framebuffer with a trailing newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", core::format_args!($($arg)*)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_info(width: usize, height: usize, format: PixelFormat) -> FrameBufferInfo {
        FrameBufferInfo {
            byte_len: width * height * 4,
            width,
            height,
            pixel_format: format,
            bytes_per_pixel: 4,
            stride: width,
        }
    }

    fn mock_writer(width: usize, height: usize, format: PixelFormat) -> FrameBufferWriter {
        let buffer: &'static mut [u8] =
            Box::leak(vec![0xAA; width * height * 4].into_boxed_slice());
        FrameBufferWriter::new(buffer, mock_info(width, height, format))
    }

    #[test]
    fn new_clears_the_screen_to_black() {
        let writer = mock_writer(64, 32, PixelFormat::Rgb);
        assert!(writer.buffer.iter().all(|byte| *byte == 0));
    }

    #[test]
    fn pixel_follows_bgr_channel_order() {
        let mut writer = mock_writer(64, 32, PixelFormat::Bgr);
        writer.write_pixel(
            0,
            0,
            Color {
                red: 1,
                green: 2,
                blue: 3,
            },
        );
        assert_eq!(&writer.buffer[..4], &[3, 2, 1, 0]);
    }

    #[test]
    fn pixel_follows_rgb_channel_order() {
        let mut writer = mock_writer(64, 32, PixelFormat::Rgb);
        writer.write_pixel(
            0,
            0,
            Color {
                red: 1,
                green: 2,
                blue: 3,
            },
        );
        assert_eq!(&writer.buffer[..4], &[1, 2, 3, 0]);
    }

    #[test]
    fn text_renders_nonblank_pixels() {
        use fmt::Write;
        let mut writer = mock_writer(64, 32, PixelFormat::Rgb);
        writer.write_str("Hi").unwrap();
        assert!(writer.buffer.iter().any(|byte| *byte != 0));
    }

    #[test]
    fn scroll_keeps_the_last_line_visible() {
        use fmt::Write;
        // 16x32 screen: two columns (no auto-wrap) and two text rows.
        let mut writer = mock_writer(16, 32, PixelFormat::Rgb);
        writer.write_str(" \nB\n").unwrap();
        // Reference: a lone 'B' rendered at the top of an empty screen.
        let mut reference = mock_writer(16, 32, PixelFormat::Rgb);
        reference.write_str("B").unwrap();
        assert_eq!(
            &writer.buffer[..16 * 16 * 4],
            &reference.buffer[..16 * 16 * 4]
        );
        assert!(writer.buffer[16 * 16 * 4..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn out_of_bounds_pixels_are_ignored() {
        let info = FrameBufferInfo {
            byte_len: 16 * 16 * 4,
            width: 8,
            height: 16,
            pixel_format: PixelFormat::Rgb,
            bytes_per_pixel: 4,
            stride: 16,
        };
        let buffer: &'static mut [u8] = Box::leak(vec![0; 16 * 16 * 4].into_boxed_slice());
        let mut writer = FrameBufferWriter::new(buffer, info);
        writer.write_pixel(8, 0, Color::WHITE);
        writer.write_pixel(0, 16, Color::WHITE);
        assert!(writer.buffer.iter().all(|byte| *byte == 0));
        writer.write_pixel(7, 0, Color::WHITE);
        assert!(writer.buffer.iter().any(|byte| *byte != 0));
    }

    #[test]
    fn truncated_buffer_never_panics() {
        let info = FrameBufferInfo {
            byte_len: 0,
            width: 64,
            height: 32,
            pixel_format: PixelFormat::Rgb,
            bytes_per_pixel: 4,
            stride: 64,
        };
        let buffer: &'static mut [u8] = Box::leak(Vec::new().into_boxed_slice());
        let mut writer = FrameBufferWriter {
            buffer,
            info,
            column: 0,
            row: 0,
            foreground: Color::WHITE,
            background: Color::BLACK,
        };
        writer.write_pixel(0, 0, Color::WHITE);
    }
}
