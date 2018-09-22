extern crate pancurses;
extern crate json;

use pancurses::*;
use std::fs::File;
use self::json::*;
use std::io::prelude::*;
use notes;
use actions::*;
use std::env;

pub struct NoteManager {
    pub root: notes::Entry,
    title: String,
    pub window: Window,
    pub cursor: i32,
    pub next_id: u32,
    pub actions: ActionList,
    pub selected: Option<u32>,
    scroll: i32,
    pub running: bool,
    file: String,
}

impl NoteManager {
    pub fn new(title: String, file: String) -> NoteManager {
        let mut root = notes::EntryBuilder::new(0).set_text("root").set_is_category(true).set_full_file_location(format!("{}/", env::current_dir().unwrap().to_str().unwrap())).set_child_indent_depth(0).set_is_open(false).build();
        root.toggle_open();

        //Create and set up the pancurses window
        let window = initscr();
        if has_colors() {
            start_color(); 
        }

        init_pair(1 as i16, COLOR_WHITE, COLOR_BLACK);
        init_pair(2 as i16, COLOR_WHITE, COLOR_BLUE);

        curs_set(0);

        window.attrset(COLOR_PAIR(1 | A_BOLD));

        window.mv(0, 0);

        window.refresh();
        window.keypad(true);
        noecho();

        let mut nm = NoteManager {
            root: root,
            title: title,
            window: window,
            cursor: 0,
            next_id: 1,
            actions: ActionList::new(),
            selected: None,
            scroll: 0,
            running: true,
            file: file.clone(),
        };

        nm.load_from_file(file.as_str());
        nm.root.child_indent_depth = 0;

        nm
    }    

    pub fn move_cursor(&mut self, amt: i32) {
        let cursor_max = self.root.flatten_children().len() as i32;

        if self.cursor + amt >= 0 && self.cursor + amt < cursor_max {
            self.unhighlight_line(self.cursor - self.scroll);

            self.cursor += amt; 

            if self.cursor < self.scroll {
                self.scroll -= 1; 
                self.render();
            } else if self.cursor >= self.scroll + self.window.get_max_y() - 2 {
                self.scroll += 1; 
                self.render();
            }

            self.highlight_line(self.cursor - self.scroll);        

        }
    }

    fn unhighlight_line(&self, line: i32) {
        self.window.mv(line + 2, 0);
        
        self.window.chgat(-1, A_COLOR, 1);
    }

    fn highlight_line(&self, line: i32) {
        self.window.mv(line + 2, 0);

        self.window.chgat(-1, A_COLOR, 2);
    }

    pub fn get_input(&self) -> Option<String> {
        let mut input = String::new();

        curs_set(1);

        let mut ch = self.window.getch();

        while ch != Some(Input::Character('\n')) {
            match ch {
                Some(Input::Character('\t')) => (),
                Some(Input::Character(c)) => {
                    if c == (127 as char) {
                        if !input.is_empty() {
                            self.window.mv(self.window.get_cur_y(), self.window.get_cur_x() - 1); 
                            self.window.delch();

                            let index = input.len() - 1;
                            input.remove(index);     
                        }
                    }
                    else if c == (27 as char) {
                        input.clear();
                        break;
                    } else {
                        input.push(c); 
                        self.window.addch(c);
                    } 
                }
                _ => ()
            }

            ch = self.window.getch();
        }
        
        curs_set(0);

        self.render();

        if input.is_empty() {
            None 
        } else {
            Some(input)
        }
    }

    pub fn get_text_input(&self, prompt: &str) -> Option<String> {
        self.display_message(format!("{}: ", prompt).as_str());

        self.get_input()
    }

    pub fn get_bool_input(&self, prompt: &str, default: bool) -> bool {
        let mut prompt_confirm = "[y/N]";
        if default == true {
            prompt_confirm = "[Y/n]";
        }

        self.display_message(format!("{} {}: ", prompt, prompt_confirm).as_str());

        match self.get_input() {
            Some(input) => {
                if (default == true && input.as_str().to_lowercase() == "n") || (default == false && input.as_str().to_lowercase() == "y") {
                    return !default; 
                } else {
                    return default; 
                }
            }, 
            None => return default
        }
    }

    pub fn display_message(&self, msg: &str) {
        self.window.mv(1, 0);
        self.window.clrtoeol();
        self.window.addstr(msg);
    }

    pub fn render(&self) {
        self.window.mv(0, 0);
        self.window.clear();

        self.window.addstr(&self.title);
        self.root.render_children(&self.window, 0, 1, self.scroll); 
        if self.root.flatten_children().len() > 0 {
            self.highlight_line(self.cursor - self.scroll);
        }
    }

    pub fn load_from_file(&mut self, file_name: &str) {
        match File::open(file_name) {
            Ok(mut file) => {
                let mut contents = String::new();

                match file.read_to_string(&mut contents) {
                    Ok(_) => {
                        let (mut root, next_id) = match json::parse(contents.as_str()) {
                            Ok(contents_json) => (notes::Entry::build_entry_from_json(&contents_json["root"]), contents_json["next_id"].as_u32().unwrap()),
                            Err(_) => (notes::EntryBuilder::new(0).set_text("root").set_is_category(true).set_full_file_location(format!("{}/", env::current_dir().unwrap().to_str().unwrap())).set_child_indent_depth(0).set_is_open(false).build(), 1)
                        }; 

                        root.set_is_open(true);

                        self.root = root;
                        self.next_id = next_id;
                    },
                    Err(_) => ()
                }
            },
            Err(_) => println!("No note file found. One will be created")
        }
    }

    pub fn write_json_to_file(&self) {
        let json_output = object! {
            "next_id" => self.next_id,
            "root" => self.root.get_as_json(),
        };

        let file_contents = json::stringify_pretty(json_output, 2);
        
        let mut file = File::create(self.file.as_str()).unwrap();

        file.write_all(file_contents.as_bytes());
    }

    pub fn end(&mut self) {
        endwin(); 

        self.write_json_to_file();
        self.running = false;
    }
}
