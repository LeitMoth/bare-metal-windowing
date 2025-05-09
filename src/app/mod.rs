use editor::TextEditor;
use explorer::Explorer;
use script::RunningScript;
use simple_interp::{ArrayString, Interpreter};
use window::Window;

use crate::{FsType, MAX_FILE_BYTES};
use core::fmt::Write;

mod editor;
pub mod explorer;
mod script;
pub mod window;

// I chose to make an App enum instead of an
// App trait because this gives me a concrete type
// with a fixed size. I did not want to have
// to figure out how to make an array of different
// trait objects on bare metal, and this adds only a small
// amount of code compared to the headache that it saved.
pub enum App {
    TextEditor(TextEditor),
    Explorer(Explorer),
    RunningScript(RunningScript),
}

impl App {
    pub fn exit(&self, fs: &mut FsType) -> (Window, ArrayString<64>) {
        let mut a = ArrayString::<64>::default();
        match self {
            App::TextEditor(text_editor) => {
                if let Ok(filename) = text_editor.filename.as_str() {
                    let mut buffer = [0u8; MAX_FILE_BYTES];
                    let len = text_editor.dump(&mut buffer);

                    let full = fs
                        .open_create(filename)
                        .and_then(|fd| fs.write(fd, &buffer[..len]).and_then(|_| fs.close(fd)));

                    match full {
                        Ok(()) => (text_editor.window.clone(), a),
                        Err(e) => {
                            let _ = write!(a, "couldn't save: {e}");
                            (text_editor.window.clone(), a)
                        }
                    }
                } else {
                    let _ = write!(a, "couldn't save: bad file name");
                    (text_editor.window.clone(), a)
                }
            }
            App::Explorer(explorer) => (explorer.window.clone(), a),
            App::RunningScript(running_script) => (running_script.window.clone(), a),
        }
    }

    pub fn title(&self) -> ArrayString<64> {
        let mut a = ArrayString::<64>::default();
        match self {
            App::TextEditor(text) => {
                let _ = write!(
                    a,
                    "EDIT:{},F6 to exit",
                    text.filename.as_str().unwrap_or("INVALID_NAME")
                );
            }
            App::Explorer(_) => {
                let _ = write!(a, "(e)dit,(r)un");
            }
            App::RunningScript(script) => {
                let _ = write!(
                    a,
                    "RUN:{},F6 to exit",
                    script.filename.as_str().unwrap_or("INVALID_NAME")
                );
            }
        };
        a
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
                                explorer.name(),
                                Interpreter::new(contents),
                            )))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                'e' => {
                    let mut text = TextEditor::new(explorer.window.clone(), explorer.name());

                    let mut buf = [0u8; MAX_FILE_BYTES];
                    if let Ok(n) = explorer.read_selected(&mut buf, fs) {
                        for c in &buf[..n] {
                            let c = *c as char;
                            if c == '\n' {
                                text.newline();
                            } else {
                                text.insert_char(c);
                            }
                        }
                        Some(App::TextEditor(text))
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
