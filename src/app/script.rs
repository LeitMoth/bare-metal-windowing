use core::fmt::Write;
use pluggable_interrupt_os::vga_buffer::{Color, ColorCode};
use simple_interp::{ArrayString, InterpreterOutput, TickStatus};

use crate::{InterpType, MAX_FILENAME_BYTES};

use super::window::Window;

pub struct RunningScript {
    pub window: Window,
    pub filename: ArrayString<MAX_FILENAME_BYTES>,
    interpreter: InterpType,
    iobuffer: IOBuffer,
    status: TickStatus,
}

#[derive(Default)]
struct IOBuffer {
    buf: ArrayString<1024>,
    input_pos: Option<usize>,
}

impl IOBuffer {
    fn begin_input(&mut self) {
        self.input_pos = Some(self.buf.len())
    }

    fn end_input(&mut self) -> &[u8] {
        match self.input_pos {
            None => &[],
            Some(begin) => &self.buf.buffer_slice()[begin..self.buf.buffer_slice().len() - 1],
        }
    }

    fn input_done(&mut self) -> bool {
        match self.input_pos {
            None => false,
            Some(begin) => {
                begin + 1 < self.buf.buffer_slice().len()
                    && self.buf.buffer_slice().last() == Some(&('\n' as u8))
            }
        }
    }
}

impl InterpreterOutput for IOBuffer {
    fn print(&mut self, chars: &[u8]) {
        for c in chars {
            self.buf.push_char(*c as char);
        }
    }
}

impl RunningScript {
    pub fn new(
        window: Window,
        filename: ArrayString<MAX_FILENAME_BYTES>,
        interpreter: InterpType,
    ) -> Self {
        let outbuffer = Default::default();
        window.clear();
        Self {
            window,
            filename,
            interpreter,
            iobuffer: outbuffer,
            status: TickStatus::Continuing,
        }
    }

    // returns true if we did any work, and false if we are blocked.
    // lib.rs uses this to determine when to increment the tick
    // counts in the task manager bar on the right of the screen.
    pub fn tick(&mut self) -> bool {
        match self.status {
            TickStatus::Continuing => {
                self.status = self.interpreter.tick(&mut self.iobuffer);
                if let TickStatus::AwaitInput = self.status {
                    self.iobuffer.begin_input();
                }
                true
            }
            TickStatus::Finished => false,
            TickStatus::AwaitInput => {
                if self.iobuffer.input_done() {
                    let input = self.iobuffer.end_input();
                    let input = str::from_utf8(input).unwrap_or("");
                    match self.interpreter.provide_input(input) {
                        Ok(()) => self.status = TickStatus::Continuing,
                        Err(e) => write!(self.iobuffer.buf, "{e}").unwrap_or(()),
                    }
                }

                false
            }
        }
    }

    pub fn draw(&mut self) {
        let color = ColorCode::new(Color::LightGray, Color::Black);
        let color_inv = ColorCode::new(Color::Black, Color::LightGray);

        // calculate the offset into the buffer we need to
        // start at in order for the below code to
        // print at most window.height() lines
        let slice = self.iobuffer.buf.buffer_slice();
        let mut count = 0;
        let mut start = 0;
        let mut lasti = slice.len();
        for i in (0..slice.len()).rev() {
            if lasti - i >= self.window.width() {
                count += 1;
                lasti = i;
            }
            if slice[i] == '\n' as u8 {
                count += 1;
                lasti = i;
            }
            if count >= self.window.height() {
                start = i + 1;
                break;
            }
        }

        // dump everything from the buffer to screen, starting at start
        let mut cursor = 0;
        let pplot = |c, cursor| {
            self.window.plot(
                c,
                (cursor % self.window.width()) as u8,
                (cursor / self.window.width()) as u8,
                color,
            );
        };
        for c in &slice[start..] {
            let c = *c as char;
            if c == '\n' {
                let tmp = cursor + self.window.width();
                let tmp = tmp - tmp % self.window.width();
                for _ in cursor..tmp {
                    pplot(' ', cursor);
                    cursor += 1;
                }
            } else {
                pplot(c, cursor);
                cursor += 1;
            }
        }

        // draw cursor block
        self.window.plot(
            ' ',
            (cursor % self.window.width()) as u8,
            (cursor / self.window.width()) as u8,
            color_inv,
        );
        cursor += 1;

        // clear the rest
        while cursor < self.window.width() * self.window.height() {
            pplot(' ', cursor);
            cursor += 1;
        }
    }

    pub fn input(&mut self, c: char) {
        self.iobuffer.buf.push_char(c);
    }
}
