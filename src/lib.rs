#![no_std]

mod window;

use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    is_drawable, plot, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH,
};
use window::{TextEditor, Window};

use core::prelude::rust_2024::derive;

const TASK_MANAGER_WIDTH: usize = 10;
const WIN_REGION_WIDTH: usize = BUFFER_WIDTH - TASK_MANAGER_WIDTH;
const MAX_OPEN: usize = 16;
const BLOCK_SIZE: usize = 256;
const NUM_BLOCKS: usize = 255;
const MAX_FILE_BLOCKS: usize = 64;
const MAX_FILE_BYTES: usize = MAX_FILE_BLOCKS * BLOCK_SIZE;
const MAX_FILES_STORED: usize = 30;
const MAX_FILENAME_BYTES: usize = 10;

const WIN_WIDTH: usize = (WIN_REGION_WIDTH - 3) / 2;

// const WIDTH_LEFT: usize = (BUFFER_WIDTH - 3) / 2;
// const WIDTH_RIGHT: usize = (BUFFER_WIDTH - 3) - WIDTH_LEFT;
const WIDTH_LEFT: usize = WIN_WIDTH;
const WIDTH_RIGHT: usize = WIN_WIDTH;
const HEIGHT_UP: usize = (BUFFER_HEIGHT - 4) / 2;
const HEIGHT_DOWN: usize = (BUFFER_HEIGHT - 4) - HEIGHT_UP;

const MIDDLE_X: usize = 1 + WIDTH_LEFT;
const MIDDLE_Y: usize = 1 + 1 + HEIGHT_UP;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Active {
    TopLeft = 0,
    TopRight = 1,
    BottomLeft = 2,
    BottomRight = 3,
}

impl Active {
    fn draw_label(&self, active: bool) {
        let color = ColorCode::new(
            if active {
                Color::LightGreen
            } else {
                Color::LightGray
            },
            Color::DarkGray,
        );
        let plot2 = |x, y, c1, c2| {
            plot(c1, x, y, color);
            plot(c2, x + 1, y, color);
        };
        match self {
            Active::TopLeft => {
                plot2(MIDDLE_X / 2, 1, 'F', '1');
            }
            Active::TopRight => {
                plot2(MIDDLE_X * 3 / 2, 1, 'F', '2');
            }
            Active::BottomLeft => {
                plot2(MIDDLE_X / 2, MIDDLE_Y, 'F', '3');
            }
            Active::BottomRight => {
                plot2(MIDDLE_X * 3 / 2, MIDDLE_Y, 'F', '4');
            }
        }
    }

    fn draw(&self, active: bool) {
        let (x1, y1, x2, y2) = match self {
            Active::TopLeft => (0, 1, MIDDLE_X, MIDDLE_Y),
            Active::TopRight => (MIDDLE_X, 1, WIN_REGION_WIDTH - 2, MIDDLE_Y),
            Active::BottomLeft => (0, MIDDLE_Y, MIDDLE_X, BUFFER_HEIGHT - 1),
            Active::BottomRight => (MIDDLE_X, MIDDLE_Y, WIN_REGION_WIDTH - 2, BUFFER_HEIGHT - 1),
        };

        let color = ColorCode::new(
            if active {
                Color::LightGreen
            } else {
                Color::LightGray
            },
            Color::DarkGray,
        );

        for col in x1..=x2 {
            plot(0xC4 as char, col, y1, color);
            plot(0xC4 as char, col, y2, color);
        }
        for row in y1..=y2 {
            plot(0xB3 as char, x1, row, color);
            plot(0xB3 as char, x2, row, color);
        }

        self.draw_label(active);

        match self {
            Active::TopLeft => {
                plot(0xDA as char, 0, 1, color);
                plot(0xC3 as char, 0, MIDDLE_Y, color);
                plot(0xC2 as char, MIDDLE_X, 1, color);
                plot(0xC5 as char, MIDDLE_X, MIDDLE_Y, color);

                // here we must redraw because we overwrote this
                Active::BottomLeft.draw_label(false);
            }
            Active::TopRight => {
                plot(0xC2 as char, MIDDLE_X, 1, color);
                plot(0xC5 as char, MIDDLE_X, MIDDLE_Y, color);
                plot(0xBF as char, WIN_REGION_WIDTH - 2, 1, color);
                plot(0xB4 as char, WIN_REGION_WIDTH - 2, MIDDLE_Y, color);

                // here we must redraw because we overwrote this
                Active::BottomRight.draw_label(false);
            }
            Active::BottomLeft => {
                plot(0xC3 as char, 0, MIDDLE_Y, color);
                plot(0xC0 as char, 0, BUFFER_HEIGHT - 1, color);
                plot(0xC5 as char, MIDDLE_X, MIDDLE_Y, color);
                plot(0xC1 as char, MIDDLE_X, BUFFER_HEIGHT - 1, color);
            }
            Active::BottomRight => {
                plot(0xC5 as char, MIDDLE_X, MIDDLE_Y, color);
                plot(0xC1 as char, MIDDLE_X, BUFFER_HEIGHT - 1, color);
                plot(0xB4 as char, WIN_REGION_WIDTH - 2, MIDDLE_Y, color);
                plot(0xD9 as char, WIN_REGION_WIDTH - 2, BUFFER_HEIGHT - 1, color);
            }
        }
    }
}

// #[derive(Copy, Clone, Eq, PartialEq)]
pub struct SwimInterface {
    active: Active,
    text_editors: [TextEditor; 4],
}

impl Default for SwimInterface {
    fn default() -> Self {
        let windows = [
            Window::new(1, 2, WIDTH_LEFT, HEIGHT_UP),
            Window::new(1 + 1 + WIDTH_LEFT, 2, WIDTH_RIGHT, HEIGHT_UP),
            Window::new(1, 2 + 1 + HEIGHT_UP, WIDTH_LEFT, HEIGHT_DOWN),
            Window::new(
                1 + 1 + WIDTH_LEFT,
                2 + 1 + HEIGHT_UP,
                WIDTH_RIGHT,
                HEIGHT_DOWN,
            ),
        ];

        Self {
            active: Active::TopLeft,
            text_editors: windows.map(TextEditor::new),
        }
    }
}

impl SwimInterface {
    pub fn init(&self) {
        [
            Active::TopLeft,
            Active::TopRight,
            Active::BottomLeft,
            Active::BottomRight,
        ]
        .map(|x| x.draw(false));

        self.active.draw(true);
    }
    pub fn tick(&mut self) {
        // self.clear_current();

        // Each TextEditor should always fill every character
        // of its window when drawn, so we never need
        // to clear anything
        self.draw_current();
    }

    // fn clear_current(&self) {
    //     clear_screen();
    // }

    // fn draw_border(&self) {
    //     let color = ColorCode::new(Color::DarkGray, Color::LightRed);
    //     for col in 0..BUFFER_WIDTH {
    //         plot(0xC4 as char, col, 0, color);
    //         plot(0xC4 as char, col, MIDDLE_Y, color);
    //         plot(0xC4 as char, col, BUFFER_HEIGHT - 1, color);
    //     }
    //     for row in 0..BUFFER_HEIGHT {
    //         plot(0xB3 as char, 0, row, color);
    //         plot(0xB3 as char, MIDDLE_X, row, color);
    //         plot(0xB3 as char, BUFFER_WIDTH - 1, row, color);
    //     }
    // }

    fn draw_current(&mut self) {
        for t in &mut self.text_editors {
            t.draw();
            // t.window.dbgdraw()
        }
    }

    pub fn key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            DecodedKey::Unicode(c) => self.handle_unicode(c),
        }
    }

    fn switch_active(&mut self, new: Active) {
        self.active.draw(false);
        self.active = new;
        self.active.draw(true);
    }

    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::F1 => self.switch_active(Active::TopLeft),
            KeyCode::F2 => self.switch_active(Active::TopRight),
            KeyCode::F3 => self.switch_active(Active::BottomLeft),
            KeyCode::F4 => self.switch_active(Active::BottomRight),
            KeyCode::ArrowLeft => self.text_editors[self.active as usize].arrow_left(),
            KeyCode::ArrowRight => self.text_editors[self.active as usize].arrow_right(),
            KeyCode::ArrowUp => self.text_editors[self.active as usize].arrow_up(),
            KeyCode::ArrowDown => self.text_editors[self.active as usize].arrow_down(),
            _ => {}
        }
    }

    fn handle_unicode(&mut self, key: char) {
        const ASCII_ENTER: char = '\n';
        const ASCII_DEL: char = '\x7F';
        const ASCII_BS: char = '\x08';

        match key {
            ASCII_ENTER => self.text_editors[self.active as usize].newline(),
            ASCII_BS | ASCII_DEL => self.text_editors[self.active as usize].backspace(),
            k if is_drawable(k) => self.text_editors[self.active as usize].insert_char(key),
            _ => {}
        }
    }
}
