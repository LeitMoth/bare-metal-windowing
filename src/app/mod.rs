use editor::TextEditor;
use explorer::Explorer;
use script::RunningScript;

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
