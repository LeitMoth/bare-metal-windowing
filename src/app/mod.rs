use editor::TextEditor;
use explorer::Explorer;
use pluggable_interrupt_os::println;
use script::RunningScript;
use simple_interp::Interpreter;
use window::Window;

use crate::{FsType, MAX_FILE_BYTES};

mod editor;
pub mod explorer;
mod script;
pub mod window;

pub enum App {
    TextEditor(TextEditor),
    Explorer(Explorer),
    RunningScript(RunningScript),
}

impl App {
    pub fn exit(&self) -> Window {
        match self {
            App::TextEditor(text_editor) => todo!("handle saving logic"),
            App::Explorer(explorer) => explorer.window.clone(),
            App::RunningScript(running_script) => running_script.window.clone(),
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            App::TextEditor(_) => "EDITING",
            App::Explorer(_) => "(e)dit\u{C4}\u{C4}(r)un",
            App::RunningScript(_) => "\u{C4}\u{C4}\u{C4}\u{C4}F6 to exit",
        }
    }
    pub fn arrow_left(&mut self) {
        match self {
            App::TextEditor(text_editor) => text_editor.arrow_left(),
            App::Explorer(explorer) => explorer.arrow_left(),
            App::RunningScript(_) => {}
        }
    }

    pub fn arrow_right(&mut self) {
        match self {
            App::TextEditor(text_editor) => text_editor.arrow_right(),
            App::Explorer(explorer) => explorer.arrow_right(),
            App::RunningScript(_) => {}
        }
    }

    pub fn arrow_up(&mut self) {
        match self {
            App::TextEditor(text_editor) => text_editor.arrow_up(),
            App::Explorer(explorer) => explorer.arrow_up(),
            App::RunningScript(_) => {}
        }
    }

    pub fn arrow_down(&mut self) {
        match self {
            App::TextEditor(text_editor) => text_editor.arrow_down(),
            App::Explorer(explorer) => explorer.arrow_down(),
            App::RunningScript(_) => {}
        }
    }

    pub fn newline(&mut self) -> Option<App> {
        match self {
            App::TextEditor(text_editor) => text_editor.newline(),
            App::Explorer(_) => {}
            App::RunningScript(running_script) => running_script.input('\n'),
        }
        None
    }

    pub fn backspace(&mut self) -> Option<App> {
        match self {
            App::TextEditor(text_editor) => text_editor.backspace(),
            App::Explorer(_) => {}
            App::RunningScript(running_script) => running_script.input('\u{8}'),
        }
        None
    }

    pub fn insert_char(&mut self, c: char, fs: &mut FsType) -> Option<App> {
        match self {
            App::TextEditor(text_editor) => {
                text_editor.insert_char(c);
                None
            }
            App::Explorer(explorer) => match c {
                'r' => {
                    let mut buf = [0u8; MAX_FILE_BYTES];
                    if let Ok(n) = explorer.read_selected(&mut buf, fs) {
                        if let Ok(contents) = str::from_utf8(&buf[..n]) {
                            Some(App::RunningScript(RunningScript::new(
                                explorer.window.clone(),
                                Interpreter::new(contents),
                            )))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            },
            App::RunningScript(running_script) => {
                running_script.input(c);
                None
            }
        }
    }

    pub fn draw(&mut self) {
        match self {
            App::TextEditor(text_editor) => text_editor.draw(),
            App::Explorer(explorer) => explorer.draw(),
            App::RunningScript(running_script) => running_script.draw(),
        }
    }
}
