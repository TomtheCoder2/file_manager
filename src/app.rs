use std::fs::ReadDir;
use std::path::PathBuf;

use crate::stateful_list::StatefulList;
use crate::{InputMode, HOME_DIR};

pub struct App {
    pub list: StatefulList<PathBuf>,
    pub error: Option<String>,
    pub input_mode: InputMode,
    pub input: String,
    pub show_popup: bool,
    pub callback: Option<Box<dyn Fn(String)>>,
}

impl App {
    pub fn new() -> App {
        let curr_directory: &str = unsafe { HOME_DIR };
        let content: ReadDir = std::fs::read_dir(curr_directory).unwrap();
        let mut items = Vec::new();
        for entry in content {
            items.push(entry.expect("ERROR").path());
        }
        App {
            list: StatefulList::with_items(items, curr_directory.to_string()),
            error: None,
            input_mode: InputMode::Normal,
            input: String::new(),
            show_popup: false,
            callback: None,
        }
    }

    pub fn on_tick(&mut self) {
        // for now nothing
    }

    pub fn go_back(&mut self) {
        let mut path = PathBuf::from(&self.list.curr_dir);
        let path_before = path.clone();
        path.pop();
        self.list.curr_dir = path.to_str().unwrap().to_string();
        let content: ReadDir = std::fs::read_dir(&self.list.curr_dir).unwrap();
        let mut items = Vec::new();
        for entry in content {
            items.push(entry.expect("ERROR").path());
        }
        self.list.items = items;
        // and select the one where we were
        let mut i = 0;
        for item in self.list.items.iter() {
            if item.to_str().unwrap() == path_before.to_str().unwrap() {
                self.list.state.select(Some(i));
                break;
            }
            i += 1;
        }
    }

    pub fn go_into(&mut self) {
        let mut path = PathBuf::from(&self.list.curr_dir);
        path.push(
            self.list.items[self.list.state.selected().unwrap()]
                .file_name()
                .unwrap(),
        );
        self.list.curr_dir = path.to_str().unwrap().to_string();
        let content: ReadDir = match std::fs::read_dir(&self.list.curr_dir) {
            Ok(content) => content,
            Err(_) => {
                // There was a major error, so we go back and show the error
                self.go_back();
                return;
            }
        };
        let mut items = Vec::new();
        for entry in content {
            items.push(entry.expect("ERROR").path());
        }
        self.list.items = items;
        // and select the first one
        self.list.state.select(Some(0));
    }

    pub fn new_folder(&mut self) {
        self.show_popup = true;
        self.input_mode = InputMode::Editing;
        let path = PathBuf::from(&self.list.curr_dir).clone();
        self.callback = Some(Box::new(move |s| {
            let mut path = path.clone();
            path.push(s);
            match std::fs::create_dir(path) {
                Ok(_) => {}
                Err(_e) => {
                    // self.error = Some(e.to_string());
                }
            }
            // self.go_into();
        }));
    }
}
