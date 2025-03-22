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
// const DOC_LINES: usize = 5;

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
    }

    pub fn newline(&mut self) {
        if self.cursor.line + 1 >= DOC_LINES {
            return;
        }

        let end = DOC_LINES - 1;
        let start = self.cursor.line;
        for i in (start + 2..=end).rev() {
            self.doc[i] = self.doc[i - 1];
        }

        self.doc[self.cursor.line + 1] = Default::default();
        for i in self.cursor.col..self.doc[self.cursor.line].len {
            self.doc[self.cursor.line + 1].data[i - self.cursor.col] =
                self.doc[self.cursor.line].data[i];
            self.doc[self.cursor.line + 1].len += 1;

            self.doc[self.cursor.line].data[i] = ' ';
            self.doc[self.cursor.line].len -= 1;
        }

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
    // fn reverse_lookup(&self, x: usize, y: usize) -> (usize, usize) {
    //     let row = self.scroll;
    //
    //     let line = 0;
    //
    //     loop {
    //         self.doc[line].len
    //     }
    //
    //     for l in &self.doc {
    //         l.
    //     }
    //
    //     let row = row + count_wrapped_until(row);
    //
    //     let x = return (y, x);
    // }

    // returns the number of rows that it took
    fn drawline_bottom(&self, line: usize, bottom_target: i64) -> i64 {
        let gray = ColorCode::new(Color::LightGray, Color::Black);
        let gray_inv = ColorCode::new(Color::Black, Color::LightGray);

        let rows_needed = (self.doc[line].len / self.window.width() + 1) as i64;
        let len = rows_needed * self.window.width() as i64;

        for i in 0..len {
            let y = i / self.window.width() as i64;
            let x = i % self.window.width() as i64;

            let c = *self.doc[line].data.get(i as usize).unwrap_or(&' ');

            let row = bottom_target - (rows_needed - 1) + y;
            if row < 0 {
                break;
            }
            self.window.plot(
                c,
                x as u8,
                row as u8,
                if line == self.cursor.line && i == self.cursor.col as i64 {
                    gray_inv
                } else {
                    gray
                },
            );
        }

        rows_needed as i64
    }

    fn keep_cursor_on_screen(&mut self) {
        let mut total = 0;
        for line in self.scroll..=self.cursor.line {
            let rows_needed = if line == self.cursor.line {
                self.cursor.col / self.window.width() + 1
            } else {
                self.doc[line].len / self.window.width() + 1
            };

            total += rows_needed;
        }

        if total == 0 {
            self.scroll = self.cursor.line;
        } else if total >= self.window.height() {
            self.scroll += total - self.window.height()
        }
    }

    fn drawline(&self, line: usize, y_base: usize) -> usize {
        let gray = ColorCode::new(Color::LightGray, Color::Black);
        let gray_inv = ColorCode::new(Color::Black, Color::LightGray);

        let rows_needed = self.doc[line].len / self.window.width() + 1;
        let len = rows_needed * self.window.width();

        for i in 0..len {
            let y = i / self.window.width() + y_base;
            let x = i % self.window.width();

            let c = *self.doc[line].data.get(i as usize).unwrap_or(&' ');

            if y >= self.window.height() {
                break;
            }
            self.window.plot(
                c,
                x as u8,
                y as u8,
                if line == self.cursor.line && i == self.cursor.col {
                    gray_inv
                } else {
                    gray
                },
            );
        }

        rows_needed
    }

    pub fn draw(&mut self) {
        self.keep_cursor_on_screen();

        // let gray = ColorCode::new(Color::LightGray, Color::Black);
        // let gray_inv = ColorCode::new(Color::Black, Color::LightGray);

        let mut used = 0;
        let mut line = self.scroll;

        while used < self.window.height() {
            used += self.drawline(line, used);
            line += 1
        }
        // let mut space = self.window.height() as i64;
        //
        // let mut bottom_line = (self.scroll + self.window.height()) as i64;
        //
        // while space > 0 && bottom_line >= 0 {
        //     space -= self.drawline_bottom(bottom_line as usize, space - 1);
        //     bottom_line -= 1
        // }
        //
        // let mut y = 0;
        // while y < self.window.height() {
        //     let line = y + self.scroll;
        //     for col in 0..=self.doc[line].len {
        //         let x = col % self.window.width();
        //         if col > 0 && x == 0 {
        //             y += 1;
        //         }
        //
        //         let c = self.doc[line].data[col];
        //
        //         self.window.plot(
        //             c,
        //             x as u8,
        //             y as u8,
        //             if line == self.cursor.line && col == self.cursor.col {
        //                 gray_inv
        //             } else {
        //                 gray
        //             },
        //         );
        //     }
        //     for x in (self.doc[line].len + 1) % self.window.width()..self.window.width() {
        //         self.window.plot(' ', x as u8, y as u8, gray);
        //     }
        //
        //     y += 1;
        // }
        // for y in 0..self.window.height() {
        //     let line = y + self.scroll;
        // }
        // for y in 0..self.window.height() {
        //     for x in 0..self.window.width() {
        //         let (line, col) = self.reverse_lookup(x, y);
        //
        //         if line >= DOC_LINES {
        //             continue;
        //         }
        //
        //         let mut c = self.doc[line].data[col];
        //
        //         if self.doc[line].len == x {
        //             c = '#'
        //         };
        //
        //         self.window.plot(
        //             c,
        //             x as u8,
        //             y as u8,
        //             if line == self.cursor.line && col == self.cursor.col {
        //                 gray_inv
        //             } else {
        //                 gray
        //             },
        //         );
        //     }
        // }
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
