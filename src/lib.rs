pub mod app;
pub mod stateful_list;
pub static mut HOME_DIR: &str = "C:\\Users\\janwi\\rust\\file_manager\\test\\";

pub enum InputMode {
    Normal,
    Editing,
}
