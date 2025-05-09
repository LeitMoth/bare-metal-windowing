#![no_std]

mod app;

use app::{explorer::Explorer, window::Window, App};
use file_system_solution::FileSystem;
use gc_heap::GenerationalHeap;
use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    is_drawable, plot, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH,
};
use ramdisk::RamDisk;
use simple_interp::{ArrayString, Interpreter};

use core::{fmt::Write, prelude::rust_2024::derive};

const MAX_TOKENS: usize = 100;
const MAX_LITERAL_CHARS: usize = 15;
const STACK_DEPTH: usize = 20;
const MAX_LOCAL_VARS: usize = 10;
const HEAP_SIZE: usize = 256;
const MAX_HEAP_BLOCKS: usize = HEAP_SIZE;

type InterpType = Interpreter<
    MAX_TOKENS,
    MAX_LITERAL_CHARS,
    STACK_DEPTH,
    MAX_LOCAL_VARS,
    WIN_WIDTH,
    GenerationalHeap<HEAP_SIZE, MAX_HEAP_BLOCKS, 2>,
>;

const TASK_MANAGER_WIDTH: usize = 10;
const WIN_REGION_WIDTH: usize = BUFFER_WIDTH - TASK_MANAGER_WIDTH;
const MAX_OPEN: usize = 16;
const BLOCK_SIZE: usize = 256;
const NUM_BLOCKS: usize = 255;
const MAX_FILE_BLOCKS: usize = 64;
const MAX_FILE_BYTES: usize = MAX_FILE_BLOCKS * BLOCK_SIZE;
const MAX_FILES_STORED: usize = 30;
const MAX_FILENAME_BYTES: usize = 10;

type FsType = FileSystem<
    MAX_OPEN,
    BLOCK_SIZE,
    NUM_BLOCKS,
    MAX_FILE_BLOCKS,
    MAX_FILE_BYTES,
    MAX_FILES_STORED,
    MAX_FILENAME_BYTES,
>;

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

fn plots(s: &str, x: usize, y: usize, limit: Option<usize>, color: ColorCode) {
    s.chars()
        .take(limit.unwrap_or(BUFFER_WIDTH - x))
        .enumerate()
        .for_each(|(i, c)| plot(c, x + i, y, color));
}

impl Active {
    fn draw_label(&self, titles: [&str; 4], active: bool) {
        let color = ColorCode::new(
            if active {
                Color::LightGreen
            } else {
                Color::LightGray
            },
            Color::DarkGray,
        );
        match self {
            Active::TopLeft => {
                plots("F1\u{C4}\u{C4}", MIDDLE_X / 2 - 9, 1, None, color);
                plots(titles[0], MIDDLE_X / 2 - 9 + 4, 1, None, color);
            }
            Active::TopRight => {
                plots("F2\u{C4}\u{C4}", MIDDLE_X * 3 / 2 - 9, 1, None, color);
                plots(titles[1], MIDDLE_X * 3 / 2 - 9 + 4, 1, None, color);
            }
            Active::BottomLeft => {
                plots("F3\u{C4}\u{C4}", MIDDLE_X / 2 - 9, MIDDLE_Y, None, color);
                plots(titles[2], MIDDLE_X / 2 - 9 + 4, MIDDLE_Y, None, color);
            }
            Active::BottomRight => {
                plots(
                    "F4\u{C4}\u{C4}",
                    MIDDLE_X * 3 / 2 - 9,
                    MIDDLE_Y,
                    None,
                    color,
                );
                plots(titles[3], MIDDLE_X * 3 / 2 - 9 + 4, MIDDLE_Y, None, color);
            }
        }
    }

    fn draw(&self, titles: [&str; 4], active: bool) {
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

        self.draw_label(titles, active);

        match self {
            Active::TopLeft => {
                plot(0xDA as char, 0, 1, color);
                plot(0xC3 as char, 0, MIDDLE_Y, color);
                plot(0xC2 as char, MIDDLE_X, 1, color);
                plot(0xC5 as char, MIDDLE_X, MIDDLE_Y, color);

                // here we must redraw because we overwrote this
                Active::BottomLeft.draw_label(titles, false);
            }
            Active::TopRight => {
                plot(0xC2 as char, MIDDLE_X, 1, color);
                plot(0xC5 as char, MIDDLE_X, MIDDLE_Y, color);
                plot(0xBF as char, WIN_REGION_WIDTH - 2, 1, color);
                plot(0xB4 as char, WIN_REGION_WIDTH - 2, MIDDLE_Y, color);

                // here we must redraw because we overwrote this
                Active::BottomRight.draw_label(titles, false);
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

pub struct SwimInterface {
    rename_bar: RenameBar,
    editing_name: bool,
    task_manager: TaskManager,
    file_system: FsType,
    active: Active,
    apps: [App; 4],
    ticks: [usize; 4],
    last_ticked: usize,
}

struct TaskManager;

struct RenameBar {
    name: ArrayString<64>,
}

impl TaskManager {
    fn draw(&self, ticks: &[usize; 4]) {
        let color = ColorCode::new(Color::LightGray, Color::Black);
        let color_alt = ColorCode::new(Color::LightGreen, Color::Black);
        let plot2 = |x, y, c1, c2| {
            plot(c1, x, y, color);
            plot(c2, x + 1, y, color);
        };
        let mut numbuf = ArrayString::<8>::default();
        plot2(WIN_REGION_WIDTH, 0, 'F', '1');
        plot(0xC0 as char, WIN_REGION_WIDTH, 1, color);
        write!(numbuf, "{}", ticks[0]).unwrap_or(());
        plots(
            numbuf.as_str().unwrap_or("ERR"),
            WIN_REGION_WIDTH + 1,
            1,
            Some(4),
            color_alt,
        );
        plot2(WIN_REGION_WIDTH, 2, 'F', '2');
        plot(0xC0 as char, WIN_REGION_WIDTH, 3, color);
        numbuf.clear();
        write!(numbuf, "{}", ticks[1]).unwrap_or(());
        plots(
            numbuf.as_str().unwrap_or("ERR"),
            WIN_REGION_WIDTH + 1,
            3,
            Some(4),
            color_alt,
        );
        plot2(WIN_REGION_WIDTH, 4, 'F', '3');
        plot(0xC0 as char, WIN_REGION_WIDTH, 5, color);
        numbuf.clear();
        write!(numbuf, "{}", ticks[2]).unwrap_or(());
        plots(
            numbuf.as_str().unwrap_or("ERR"),
            WIN_REGION_WIDTH + 1,
            5,
            Some(4),
            color_alt,
        );
        plot2(WIN_REGION_WIDTH, 6, 'F', '4');
        plot(0xC0 as char, WIN_REGION_WIDTH, 7, color);
        numbuf.clear();
        write!(numbuf, "{}", ticks[3]).unwrap_or(());
        plots(
            numbuf.as_str().unwrap_or("ERR"),
            WIN_REGION_WIDTH + 1,
            7,
            Some(4),
            color_alt,
        );
    }
}

impl RenameBar {
    fn draw(&self, active: bool) {
        let color = ColorCode::new(Color::LightGray, Color::Black);
        let green = ColorCode::new(Color::LightGreen, Color::Black);
        let label = "F5 - Filename: ";
        plots(label, 0, 0, None, if active { green } else { color });
        let name = self.name.as_str().unwrap_or("");
        plots(name, label.len(), 0, None, color);
        for i in label.len() + name.len()..WIN_REGION_WIDTH {
            plot(' ', i, 0, color);
        }
    }
}

impl Default for SwimInterface {
    fn default() -> Self {
        let w_top_left = Window::new(1, 2, WIDTH_LEFT, HEIGHT_UP);
        let w_top_right = Window::new(1 + 1 + WIDTH_LEFT, 2, WIDTH_RIGHT, HEIGHT_UP);
        let w_bottom_left = Window::new(1, 2 + 1 + HEIGHT_UP, WIDTH_LEFT, HEIGHT_DOWN);
        let w_bottom_right = Window::new(
            1 + 1 + WIDTH_LEFT,
            2 + 1 + HEIGHT_UP,
            WIDTH_RIGHT,
            HEIGHT_DOWN,
        );

        let rd = RamDisk::<BLOCK_SIZE, NUM_BLOCKS>::new();
        let mut file_system = FileSystem::new(rd);

        {
            let fd_hello = file_system.open_create("hello").unwrap();
            file_system
                .write(fd_hello, r#"print("Hello, world!")"#.as_bytes())
                .unwrap();
            file_system.close(fd_hello).unwrap();
        }

        {
            let fd_nums = file_system.open_create("nums").unwrap();
            file_system
                .write(
                    fd_nums,
                    r#"print(1)
print(257)"#
                        .as_bytes(),
                )
                .unwrap();
            file_system.close(fd_nums).unwrap();
        }

        {
            let fd_average = file_system.open_create("average").unwrap();
            file_system
                .write(
                    fd_average,
                    r#"sum := 0
count := 0
averaging := true
while averaging {
    num := input("Enter a number:")
    if (num == "quit") {
        averaging := false
    } else {
        sum := (sum + num)
        count := (count + 1)
    }
}
print((sum / count))"#
                        .as_bytes(),
                )
                .unwrap();
            file_system.close(fd_average).unwrap();
        }

        {
            let fd_pi = file_system.open_create("pi").unwrap();
            file_system
                .write(
                    fd_pi,
                    r#"sum := 0
i := 0
neg := false
terms := input("Num terms:")
while (i < terms) {
    term := (1.0 / ((2.0 * i) + 1.0))
    if neg {
        term := -term
    }
    sum := (sum + term)
    neg := not neg
    i := (i + 1)
}
print((4 * sum))"#
                        .as_bytes(),
                )
                .unwrap();
            file_system.close(fd_pi).unwrap();
        }

        let apps = [
            App::Explorer(Explorer::new(w_top_left, &mut file_system)),
            App::Explorer(Explorer::new(w_top_right, &mut file_system)),
            App::Explorer(Explorer::new(w_bottom_left, &mut file_system)),
            App::Explorer(Explorer::new(w_bottom_right, &mut file_system)),
        ];

        let task_manager = TaskManager;
        let rename_bar = RenameBar {
            name: Default::default(),
        };

        Self {
            rename_bar,
            editing_name: false,
            task_manager,
            file_system,
            active: Active::TopLeft,
            apps,
            ticks: [0, 0, 0, 0],
            last_ticked: 0,
        }
    }
}

impl SwimInterface {
    pub fn init(&mut self) {
        self.switch_active(Active::TopRight);
        self.switch_active(Active::BottomLeft);
        self.switch_active(Active::BottomRight);
        self.switch_active(Active::TopLeft);
    }

    pub fn tick(&mut self) {
        // self.clear_current();

        // Round robin, finds first app that is a script
        // and runs it, if no apps are scripts, it gives up.
        for _ in 1..4 {
            self.last_ticked += 1;
            self.last_ticked %= 4;
            match self.apps[self.last_ticked] {
                App::RunningScript(ref mut running_script) => {
                    if running_script.tick() {
                        self.ticks[self.last_ticked] += 1;
                        // IMPORTANT: this break mades it so that only
                        // one script ticks per overall tick
                        break;
                    }
                }
                _ => (),
            }
        }

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
        for t in &mut self.apps {
            t.draw();
            // t.window.dbgdraw()
        }
        self.rename_bar.draw(self.editing_name);
        self.task_manager.draw(&self.ticks);
    }

    pub fn key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            DecodedKey::Unicode(c) => self.handle_unicode(c),
        }
    }

    fn switch_active(&mut self, new: Active) {
        let titles = [
            self.apps[Active::TopLeft as usize].title(),
            self.apps[Active::TopRight as usize].title(),
            self.apps[Active::BottomLeft as usize].title(),
            self.apps[Active::BottomRight as usize].title(),
        ];
        self.active.draw(titles, false);
        self.active = new;
        self.active.draw(titles, true);
    }

    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::F1 => self.switch_active(Active::TopLeft),
            KeyCode::F2 => self.switch_active(Active::TopRight),
            KeyCode::F3 => self.switch_active(Active::BottomLeft),
            KeyCode::F4 => self.switch_active(Active::BottomRight),
            KeyCode::F5 => {
                self.editing_name = true;
                self.rename_bar.name.clear()
            }
            KeyCode::F6 => {
                let window = self.apps[self.active as usize].exit();
                self.apps[self.active as usize] =
                    App::Explorer(Explorer::new(window, &mut self.file_system));
                // refresh display
                self.switch_active(self.active);
            }
            KeyCode::ArrowLeft => self.apps[self.active as usize].arrow_left(),
            KeyCode::ArrowRight => self.apps[self.active as usize].arrow_right(),
            KeyCode::ArrowUp => self.apps[self.active as usize].arrow_up(),
            KeyCode::ArrowDown => self.apps[self.active as usize].arrow_down(),
            _ => {}
        }
    }

    fn handle_unicode(&mut self, key: char) {
        const ASCII_ENTER: char = '\n';
        const ASCII_DEL: char = '\x7F';
        const ASCII_BS: char = '\x08';

        if self.editing_name {
            match key {
                ASCII_ENTER => {
                    self.editing_name = false;

                    match self.rename_bar.name.as_str() {
                        Ok(name) => {
                            if name.len() < 1 {
                                self.rename_bar.name.clear();
                                let _ = write!(
                                    self.rename_bar.name,
                                    "ERROR File name must be at least one character"
                                );
                            } else {
                                match self.file_system.open_create(name) {
                                    Ok(fd) => match self.file_system.close(fd) {
                                        Ok(()) => {
                                            self.rename_bar.name.clear();
                                        }
                                        Err(e) => {
                                            self.rename_bar.name.clear();
                                            let _ = write!(self.rename_bar.name, "ERROR {e}");
                                        }
                                    },
                                    Err(e) => {
                                        self.rename_bar.name.clear();
                                        let _ = write!(self.rename_bar.name, "ERROR {e}");
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            self.rename_bar.name.clear();
                            let _ = write!(self.rename_bar.name, "ERROR {e}");
                        }
                    }

                    for i in 0..4 {
                        if let App::Explorer(ref exp) = self.apps[i] {
                            self.apps[i] = App::Explorer(Explorer::new(
                                exp.window.clone(),
                                &mut self.file_system,
                            ));
                        }
                    }
                }
                k => self.rename_bar.name.push_char(k),
            }
        } else {
            if let Some(newapp) = match key {
                ASCII_ENTER => self.apps[self.active as usize].newline(),
                ASCII_BS | ASCII_DEL => self.apps[self.active as usize].backspace(),
                k if is_drawable(k) => {
                    self.apps[self.active as usize].insert_char(key, &mut self.file_system)
                }
                _ => None,
            } {
                self.apps[self.active as usize] = newapp;
                // refresh display
                self.switch_active(self.active);
            }
        }
    }
}
