use pluggable_interrupt_os::vga_buffer::{plot, Color, ColorCode};

use crate::{FsType, MAX_FILENAME_BYTES};

#[derive(Debug, Clone)]
pub struct Window {
    pub x1: u8,
    pub y1: u8,
    pub x2: u8,
    pub y2: u8,
}

impl Window {
    // must have nonzero width and height!
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Self {
            x1: x as u8,
            y1: y as u8,
            x2: (x + w - 1) as u8,
            y2: (y + h - 1) as u8,
        }
    }

    pub fn height(&self) -> usize {
        (self.y2 - self.y1 + 1) as usize
    }
    pub fn width(&self) -> usize {
        (self.x2 - self.x1 + 1) as usize
    }

    // pub fn dbgdraw(&self) {
    //     let dbgcolor = ColorCode::new(Color::Black, Color::LightGreen);
    //     plot('#', self.x1 as usize, self.y1 as usize, dbgcolor);
    //     plot('#', self.x1 as usize, self.y2 as usize, dbgcolor);
    //     plot('#', self.x2 as usize, self.y1 as usize, dbgcolor);
    //     plot('#', self.x2 as usize, self.y2 as usize, dbgcolor);
    // }

    pub fn plot(&self, c: char, col: u8, row: u8, color: ColorCode) {
        let col = col + self.x1;
        let row = row + self.y1;
        debug_assert!(self.x1 <= col && col <= self.x2);
        debug_assert!(self.y1 <= row && row <= self.y2);
        plot(c, col as usize, row as usize, color);
    }
}
