use pluggable_interrupt_os::vga_buffer::{plot, Color, ColorCode};

use crate::{FsType, MAX_FILENAME_BYTES};

#[derive(Clone)]
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

    // pub fn dbgdraw(&self) {
    //     let dbgcolor = ColorCode::new(Color::Black, Color::LightGreen);
    //     plot('#', self.x1 as usize, self.y1 as usize, dbgcolor);
    //     plot('#', self.x1 as usize, self.y2 as usize, dbgcolor);
    //     plot('#', self.x2 as usize, self.y1 as usize, dbgcolor);
    //     plot('#', self.x2 as usize, self.y2 as usize, dbgcolor);
    // }

    fn plot(&self, c: char, col: u8, row: u8, color: ColorCode) {
        let col = col + self.x1;
        let row = row + self.y1;
        debug_assert!(self.x1 <= col && col <= self.x2);
        debug_assert!(self.y1 <= row && row <= self.y2);
        plot(c, col as usize, row as usize, color);
    }
}

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

pub enum App {
    TextEditor(TextEditor),
    Explorer(Explorer),
    RunningScript(RunningScript),
}

impl App {
    pub fn arrow_left(&mut self) {
        match self {
            App::TextEditor(text_editor) => text_editor.arrow_left(),
            App::Explorer(explorer) => explorer.arrow_left(),
            App::RunningScript(running_script) => todo!(),
        }
    }

    pub fn arrow_right(&mut self) {
        match self {
            App::TextEditor(text_editor) => text_editor.arrow_right(),
            App::Explorer(explorer) => explorer.arrow_right(),
            App::RunningScript(running_script) => todo!(),
        }
    }

    pub fn arrow_up(&mut self) {
        match self {
            App::TextEditor(text_editor) => text_editor.arrow_up(),
            App::Explorer(explorer) => explorer.arrow_up(),
            App::RunningScript(running_script) => todo!(),
        }
    }

    pub fn arrow_down(&mut self) {
        match self {
            App::TextEditor(text_editor) => text_editor.arrow_down(),
            App::Explorer(explorer) => explorer.arrow_down(),
            App::RunningScript(running_script) => todo!(),
        }
    }

    pub fn newline(&mut self) {
        match self {
            App::TextEditor(text_editor) => text_editor.newline(),
            App::Explorer(explorer) => {}
            App::RunningScript(running_script) => todo!(),
        }
    }

    pub fn backspace(&mut self) {
        match self {
            App::TextEditor(text_editor) => text_editor.backspace(),
            App::Explorer(explorer) => {}
            App::RunningScript(running_script) => todo!(),
        }
    }

    pub fn insert_char(&mut self, c: char) {
        match self {
            App::TextEditor(text_editor) => text_editor.insert_char(c),
            App::Explorer(explorer) => {} // TODO(colin): implement editing and running!!!
            App::RunningScript(running_script) => todo!(),
        }
    }

    pub fn draw(&mut self) {
        match self {
            App::TextEditor(text_editor) => text_editor.draw(),
            App::Explorer(explorer) => explorer.draw(),
            App::RunningScript(running_script) => todo!(),
        }
    }
}

pub struct Explorer {
    selected: usize,
    num_files: usize,
    names: [[u8; MAX_FILENAME_BYTES]; 3 * 10], // 3 cols, 10 rows
    window: Window,
}

impl Explorer {
    pub fn new(window: Window, fs: &mut FsType) -> Self {
        let (num_files, names) = fs.list_directory().unwrap();
        Explorer {
            selected: 0,
            num_files,
            names,
            window,
        }
    }

    fn name_as_slice(&self, i: usize) -> &[u8] {
        let mut end = MAX_FILENAME_BYTES;
        for j in 0..MAX_FILENAME_BYTES {
            if self.names[i][j] == 0 {
                end = j
            }
        }
        &self.names[i][0..end]
    }

    fn draw(&self) {
        for row in 0..10 {
            for col in 0..3 {
                for ci in 0..MAX_FILENAME_BYTES {
                    let idx = row * 3 + col;
                    let color = if idx == self.selected {
                        ColorCode::new(Color::Black, Color::LightGray)
                    } else {
                        ColorCode::new(Color::LightGray, Color::Black)
                    };

                    self.window.plot(
                        self.names[idx][ci] as char,
                        (col * MAX_FILENAME_BYTES + ci) as u8,
                        row as u8,
                        color,
                    );
                }
            }
        }
    }

    fn arrow_left(&mut self) {
        self.selected = match self.selected % 3 {
            1..3 => self.selected - 1,
            _ => self.selected,
        }
    }

    fn arrow_right(&mut self) {
        self.selected = match self.selected % 3 {
            0..2 => self.selected + 1,
            _ => self.selected,
        };
        if self.selected >= self.num_files {
            self.selected = self.num_files - 1;
        }
    }

    fn arrow_up(&mut self) {
        if self.selected >= 3 {
            self.selected -= 3;
        }
    }

    fn arrow_down(&mut self) {
        self.selected += 3;
        if self.selected >= self.num_files {
            self.selected = self.num_files - 1;
        }
    }
}

struct RunningScript;

const DOC_LINES: usize = MAX_WINDOW_ROWS * 4;
// Uncomment to test for out of bounds errors at the end of the document:
// const DOC_LINES: usize = 5;

pub struct TextEditor {
    lines: [Line; DOC_LINES],
    cursor: Cursor,
    scroll: usize,
    window: Window,
}

impl TextEditor {
    pub fn new(window: Window) -> Self {
        Self {
            lines: [Line::default(); DOC_LINES],
            cursor: Cursor { line: 0, col: 0 },
            scroll: 0,
            window,
        }
    }
    fn sanity(&mut self) {
        if self.cursor.col > self.lines[self.cursor.line].len {
            self.cursor.col = self.lines[self.cursor.line].len
        }
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
