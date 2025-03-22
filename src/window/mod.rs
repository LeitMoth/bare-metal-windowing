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

    fn height(&self) -> usize {
        (self.y2 - self.y1 + 1) as usize
    }
    fn width(&self) -> usize {
        (self.x2 - self.x1 + 1) as usize
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

    pub fn insert_char(&mut self, c: char) {
        self.sanity();

        // let r = &mut self.doc[self.cursor.line].data[0..self.doc[self.cursor.line].len];
        if self.doc[self.cursor.line].len >= 255 {
            return;
        }
        if self.doc[self.cursor.line].len != self.cursor.col {
            let end = self.doc[self.cursor.line].len;
            let start = self.cursor.col + 1;
            for i in (start..=end).rev() {
                self.doc[self.cursor.line].data[i] = self.doc[self.cursor.line].data[i - 1];
            }
        }
        self.doc[self.cursor.line].len += 1;
        self.doc[self.cursor.line].data[self.cursor.col] = c;
        self.cursor.col += 1;

        // TODO(colin): update scroll here if necessary
        // mark dirty?
    }

    pub fn newline(&mut self) {
        self.cursor.line += 1;
        self.cursor.col = 0;
        if self.cursor.line >= DOC_LINES {
            self.cursor.line = DOC_LINES - 1
        }
    }

    // TODO(colin): VERY messy!
    // break out some helper line functionality to separate methods if you have time later
    pub fn backspace(&mut self) {
        if self.cursor.col == 0 {
            if self.cursor.line == 0 {
                return;
            } else {
                self.cursor.line -= 1;
                self.cursor.col = self.doc[self.cursor.line].len;

                if self.cursor.line < DOC_LINES - 1 {
                    for i in 0..self.doc[self.cursor.line + 1].len {
                        self.doc[self.cursor.line].data[self.doc[self.cursor.line].len] =
                            self.doc[self.cursor.line + 1].data[i];
                        self.doc[self.cursor.line].len += 1
                    }
                }

                for i in self.cursor.line + 1..DOC_LINES - 1 {
                    self.doc[i] = self.doc[i + 1];
                }
                self.doc[DOC_LINES - 1] = Default::default();
            }
        } else {
            self.cursor.col -= 1;
            for i in self.cursor.col..self.doc[self.cursor.line].len - 1 {
                self.doc[self.cursor.line].data[i] = self.doc[self.cursor.line].data[i + 1]
            }
            self.doc[self.cursor.line].data[self.doc[self.cursor.line].len - 1] = ' ';
            self.doc[self.cursor.line].len -= 1;
        }
    }

    // take x,y in buffer, find data line, col
    fn reverse_lookup(&self, x: usize, y: usize) -> (usize, usize) {
        return (y, x);
    }

    pub fn draw(&self) {
        let gray = ColorCode::new(Color::LightGray, Color::Black);
        let gray_inv = ColorCode::new(Color::Black, Color::LightGray);
        for y in 0..self.window.height() {
            for x in 0..self.window.width() {
                let (line, col) = self.reverse_lookup(x, y);

                let mut c = self.doc[line].data[col];

                if self.doc[line].len == x {
                    c = '#'
                };

                self.window.plot(
                    c,
                    x as u8,
                    y as u8,
                    if line == self.cursor.line && col == self.cursor.col {
                        gray_inv
                    } else {
                        gray
                    },
                );
            }
        }
    }

    pub fn arrow_left(&mut self) {
        if self.cursor.col == 0 {
            if self.cursor.line == 0 {
                return;
            } else {
                self.cursor.line -= 1;
                self.cursor.col = self.doc[self.cursor.line].len
            }
        } else {
            self.cursor.col -= 1
        }
    }

    pub fn arrow_right(&mut self) {
        if self.cursor.col == self.doc[self.cursor.line].len {
            if self.cursor.line == DOC_LINES - 1 {
                return;
            } else {
                self.cursor.line += 1;
                self.cursor.col = 0
            }
        } else {
            self.cursor.col += 1
        }
    }

    pub fn arrow_up(&mut self) {
        if self.cursor.line == 0 {
            return;
        } else {
            self.cursor.line -= 1;
            self.cursor.col = usize::min(self.cursor.col, self.doc[self.cursor.line].len)
        }
    }

    pub fn arrow_down(&mut self) {
        if self.cursor.line == DOC_LINES - 1 {
            return;
        } else {
            self.cursor.line += 1;
            self.cursor.col = usize::min(self.cursor.col, self.doc[self.cursor.line].len)
        }
    }
}
