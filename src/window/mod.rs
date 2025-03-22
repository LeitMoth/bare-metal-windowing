use pluggable_interrupt_os::{
    println,
    vga_buffer::{plot, Color, ColorCode, BUFFER_WIDTH},
};

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

    pub fn dbgdraw(&self) {
        let dbgcolor = ColorCode::new(Color::Black, Color::LightGreen);
        plot('#', self.x1 as usize, self.y1 as usize, dbgcolor);
        plot('#', self.x1 as usize, self.y2 as usize, dbgcolor);
        plot('#', self.x2 as usize, self.y1 as usize, dbgcolor);
        plot('#', self.x2 as usize, self.y2 as usize, dbgcolor);
    }

    fn plot(&self, c: char, col: u8, row: u8, color: ColorCode) {
        let col = col + self.x1;
        let row = row + self.y1;
        debug_assert!(self.x1 <= col && col <= self.x2);
        debug_assert!(self.y1 <= row && row <= self.y2);
        plot(c, col as usize, row as usize, color);
    }
}

const WINDOW_ROWS: usize = 10;

#[derive(Clone, Copy)]
struct Line {
    data: [char; 256],
    len: usize,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            data: [' '; 256],
            len: 0,
        }
    }
}

struct Cursor {
    line: usize,
    col: usize,
}

const DOC_LINES: usize = WINDOW_ROWS * 4;

pub struct TextEditor {
    doc: [Line; DOC_LINES],
    cursor: Cursor,
    scroll: usize,
    pub window: Window,
}

impl TextEditor {
    pub fn new(window: Window) -> Self {
        Self {
            doc: [Line::default(); DOC_LINES],
            cursor: Cursor { line: 0, col: 0 },
            scroll: 0,
            window,
        }
    }
    fn sanity(&mut self) {
        if self.cursor.col > self.doc[self.cursor.line].len {
            self.cursor.col = self.doc[self.cursor.line].len
        }
    }

    pub fn putc(&mut self, c: char) {
        self.sanity();

        // let r = &mut self.doc[self.cursor.line].data[0..self.doc[self.cursor.line].len];
        if self.doc[self.cursor.line].len >= 255 {
            return;
        }
        self.doc[self.cursor.line].len += 1;
        self.doc[self.cursor.line].data[self.cursor.col] = c;
        self.cursor.col += 1;
        // TODO(colin): update scroll here if necessary
        // mark dirty?
    }

    pub fn draw(&self) {
        let gray = ColorCode::new(Color::LightGray, Color::Black);

        for row in 0..8 {
            for col in 0..self.doc[row].len {
                let c = self.doc[row].data[col];
                self.window.plot(c, col as u8, row as u8, gray);
            }
        }
    }
}
