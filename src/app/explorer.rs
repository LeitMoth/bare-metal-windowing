use file_system_solution::FileSystemError;
use pluggable_interrupt_os::vga_buffer::{Color, ColorCode};
use simple_interp::ArrayString;

use crate::{FsType, MAX_FILENAME_BYTES};

use super::window::Window;

pub struct Explorer {
    selected: usize,
    num_files: usize,
    names: [[u8; MAX_FILENAME_BYTES]; 3 * 10], // 3 cols, 10 rows
    pub window: Window,
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

    pub fn read_selected(
        &mut self,
        buf: &mut [u8],
        fs: &mut FsType,
    ) -> Result<usize, FileSystemError> {
        let filename = self.name_as_slice(self.selected);
        let filename = str::from_utf8(filename).map_err(|_| FileSystemError::FileNotFound)?;
        let fd = fs.open_read(filename)?;
        let n = fs.read(fd, buf)?;
        fs.close(fd)?;
        Ok(n)
    }

    pub fn name(&self) -> ArrayString<MAX_FILENAME_BYTES> {
        let mut a: ArrayString<MAX_FILENAME_BYTES> = Default::default();

        for c in self.name_as_slice(self.selected) {
            a.push_char(*c as char);
        }

        a
    }

    fn name_as_slice(&self, i: usize) -> &[u8] {
        let mut end = MAX_FILENAME_BYTES;
        for j in 0..MAX_FILENAME_BYTES {
            if self.names[i][j] == 0 {
                end = j;
                break;
            }
        }
        &self.names[i][0..end]
    }

    pub fn draw(&self) {
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

    pub fn arrow_left(&mut self) {
        self.selected = match self.selected % 3 {
            1..3 => self.selected - 1,
            _ => self.selected,
        }
    }

    pub fn arrow_right(&mut self) {
        self.selected = match self.selected % 3 {
            0..2 => self.selected + 1,
            _ => self.selected,
        };
        if self.selected >= self.num_files {
            self.selected = self.num_files - 1;
        }
    }

    pub fn arrow_up(&mut self) {
        if self.selected >= 3 {
            self.selected -= 3;
        }
    }

    pub fn arrow_down(&mut self) {
        self.selected += 3;
        if self.selected >= self.num_files {
            self.selected = self.num_files - 1;
        }
    }
}
