use pluggable_interrupt_os::vga_buffer::{Color, ColorCode};
use simple_interp::ArrayString;

use crate::MAX_FILENAME_BYTES;

use super::window::Window;

// Overestimate, so we have plenty of room for lines when we multiply this by 4.
// Window::height should be used to get the actuall number of rows
const MAX_WINDOW_ROWS: usize = 16;

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

const DOC_LINES: usize = MAX_WINDOW_ROWS * 4;
// Uncomment to test for out of bounds errors at the end of the document:
// const DOC_LINES: usize = 5;

pub struct TextEditor {
    lines: [Line; DOC_LINES],
    cursor: Cursor,
    scroll: usize,
    pub window: Window,
    pub filename: ArrayString<MAX_FILENAME_BYTES>,
}

impl TextEditor {
    pub fn new(window: Window, filename: ArrayString<MAX_FILENAME_BYTES>) -> Self {
        Self {
            lines: [Line::default(); DOC_LINES],
            cursor: Cursor { line: 0, col: 0 },
            scroll: 0,
            window,
            filename,
        }
    }
    fn sanity(&mut self) {
        if self.cursor.col > self.lines[self.cursor.line].len {
            self.cursor.col = self.lines[self.cursor.line].len
        }
    }

    pub fn dump(&self, buf: &mut [u8]) -> usize {
        let mut j = 0;
        for i in 0..self.lines.len() {
            for c in &self.lines[i].data[..self.lines[i].len] {
                let c = *c as u8;
                if j >= buf.len() {
                    return j;
                } else {
                    buf[j] = c;
                    j += 1;
                }
            }
            if j >= buf.len() {
                return j;
            } else {
                buf[j] = '\n' as u8;
                j += 1;
            }
        }

        while j > 1 && buf[j - 1] == '\n' as u8 {
            j -= 1;
        }

        j
    }

    pub fn insert_char(&mut self, c: char) {
        self.sanity();

        if self.lines[self.cursor.line].len >= 255 {
            return;
        }
        if self.lines[self.cursor.line].len != self.cursor.col {
            let end = self.lines[self.cursor.line].len;
            let start = self.cursor.col + 1;
            for i in (start..=end).rev() {
                self.lines[self.cursor.line].data[i] = self.lines[self.cursor.line].data[i - 1];
            }
        }
        self.lines[self.cursor.line].len += 1;
        self.lines[self.cursor.line].data[self.cursor.col] = c;
        self.cursor.col += 1;
    }

    pub fn newline(&mut self) {
        if self.cursor.line + 1 >= DOC_LINES {
            return;
        }

        let end = DOC_LINES - 1;
        let start = self.cursor.line;
        for i in (start + 2..=end).rev() {
            self.lines[i] = self.lines[i - 1];
        }

        self.lines[self.cursor.line + 1] = Default::default();
        for i in self.cursor.col..self.lines[self.cursor.line].len {
            self.lines[self.cursor.line + 1].data[i - self.cursor.col] =
                self.lines[self.cursor.line].data[i];
            self.lines[self.cursor.line + 1].len += 1;

            self.lines[self.cursor.line].data[i] = ' ';
            self.lines[self.cursor.line].len -= 1;
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
                self.cursor.col = self.lines[self.cursor.line].len;

                if self.cursor.line < DOC_LINES - 1 {
                    for i in 0..self.lines[self.cursor.line + 1].len {
                        self.lines[self.cursor.line].data[self.lines[self.cursor.line].len] =
                            self.lines[self.cursor.line + 1].data[i];
                        self.lines[self.cursor.line].len += 1
                    }
                }

                for i in self.cursor.line + 1..DOC_LINES - 1 {
                    self.lines[i] = self.lines[i + 1];
                }
                self.lines[DOC_LINES - 1] = Default::default();
            }
        } else {
            self.cursor.col -= 1;
            for i in self.cursor.col..self.lines[self.cursor.line].len - 1 {
                self.lines[self.cursor.line].data[i] = self.lines[self.cursor.line].data[i + 1]
            }
            self.lines[self.cursor.line].data[self.lines[self.cursor.line].len - 1] = ' ';
            self.lines[self.cursor.line].len -= 1;
        }
    }

    fn keep_cursor_on_screen(&mut self) {
        let mut total = 0;
        for line in self.scroll..=self.cursor.line {
            let rows_needed = if line == self.cursor.line {
                self.cursor.col / self.window.width() + 1
            } else {
                self.lines[line].len / self.window.width() + 1
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

        let rows_needed = self.lines[line].len / self.window.width() + 1;
        let len = rows_needed * self.window.width();

        for i in 0..len {
            let y = i / self.window.width() + y_base;
            let x = i % self.window.width();

            let c = *self.lines[line].data.get(i as usize).unwrap_or(&' ');

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

    fn clear_y(&self, y: u8) {
        let gray = ColorCode::new(Color::LightGray, Color::Black);
        for col in 0..self.window.width() {
            self.window.plot(' ', col as u8, y, gray);
        }
    }

    pub fn draw(&mut self) {
        self.keep_cursor_on_screen();

        let mut used = 0;
        let mut line = self.scroll;

        while used < self.window.height() && line < DOC_LINES {
            used += self.drawline(line, used);
            line += 1
        }
        while used < self.window.height() {
            self.clear_y(used as u8);
            used += 1;
        }
    }

    pub fn arrow_left(&mut self) {
        if self.cursor.col == 0 {
            if self.cursor.line == 0 {
                return;
            } else {
                self.cursor.line -= 1;
                self.cursor.col = self.lines[self.cursor.line].len
            }
        } else {
            self.cursor.col -= 1
        }
    }

    pub fn arrow_right(&mut self) {
        if self.cursor.col == self.lines[self.cursor.line].len {
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
            self.cursor.col = usize::min(self.cursor.col, self.lines[self.cursor.line].len)
        }
    }

    pub fn arrow_down(&mut self) {
        if self.cursor.line == DOC_LINES - 1 {
            return;
        } else {
            self.cursor.line += 1;
            self.cursor.col = usize::min(self.cursor.col, self.lines[self.cursor.line].len)
        }
    }
}
