extern crate pancurses;
extern crate json;

use pancurses::*;
use self::json::*;
use std::borrow::BorrowMut;
use std::process::Command;
use chrono::prelude::*;
use note_manager;
use std::fs;

#[derive(Clone)]
pub enum SortType {
    NAME,
    FILE,
    TIME,
}

pub struct Entry {
    id: u32,
    is_category: bool,
    pub text: String,
    children: Vec<Entry>,
    pub child_indent_depth: i32,
    is_open: bool,
    pub file_location: String,
    is_deleted: bool,
    pub time_created: DateTime<Local>, 
    sort_type: SortType,
    sort_descending: bool,
}

pub struct EntryBuilder {
    id: u32,
    is_category: bool,
    text: String,
    children: Vec<Entry>,
    child_indent_depth: i32,
    file_location: String,
    time_created: DateTime<Local>,
    sort_type: SortType,
    sort_descending: bool,
    open: bool,
}

impl EntryBuilder {
    pub fn new(id: u32) -> EntryBuilder {
        EntryBuilder {
            id: id,
            is_category: false,
            text: String::new(),
            children: Vec::new(),
            child_indent_depth: 4,
            file_location: String::from(""),
            time_created: Local::now(),
            sort_type: SortType::NAME,
            sort_descending: false,
            open: false,
        } 
    }

    pub fn set_text(mut self, text: &str) -> EntryBuilder {
        self.text = String::from(text);

        self
    }

    pub fn set_child_indent_depth(mut self, new_child_indent_depth: i32) -> EntryBuilder {
        self.child_indent_depth = new_child_indent_depth;

        self
    } 

    pub fn set_file_location(mut self, new_file_location: String, nm: &note_manager::NoteManager, category_id: u32) -> EntryBuilder {
        self.file_location = build_full_file_path(new_file_location, nm, category_id, self.is_category);

        self
    }

    pub fn set_full_file_location(mut self, new_file_location: String) -> EntryBuilder {
        self.file_location = new_file_location;

        self
    }

    pub fn set_time_created(mut self, new_time_created: i64) -> EntryBuilder {
        self.time_created = Local.timestamp(new_time_created, 0);

        self
    }

    pub fn set_sort_type(mut self, sort_type: SortType) -> EntryBuilder {
        self.sort_type = sort_type;

        self
    }

    pub fn set_sort_descending(mut self, sort_descending: bool) -> EntryBuilder {
        self.sort_descending = sort_descending;

        self
    }

    pub fn set_is_category(mut self, is_category: bool) -> EntryBuilder {
        self.is_category = is_category;

        self
    }

    pub fn set_is_open(mut self, is_open: bool) -> EntryBuilder {
        self.open = is_open;

        self
    }

    pub fn build(self) -> Entry {
        if self.is_category {
            fs::create_dir_all(self.file_location.as_str()); 
        } else {
            fs::create_dir_all(&self.file_location[0..self.file_location.rfind('/').unwrap()]);
        }

        Entry {
            id: self.id,
            is_category: self.is_category,
            text: self.text,
            children: self.children,
            child_indent_depth: self.child_indent_depth,
            is_open: self.open,
            file_location: self.file_location,
            is_deleted: false,
            time_created: self.time_created,
            sort_type: self.sort_type,
            sort_descending: self.sort_descending,
        } 
    }
}

impl Entry {
    pub fn add_child(&mut self, new_child: Entry) {
        self.children.push(new_child); 
        self.sort_children();
    }

    pub fn can_add_child(&self) -> bool {
        self.is_category
    }

    pub fn render_entry(&self, window: &Window, x: i32, y: i32, scroll: i32) {
        if y < window.get_max_y() + scroll && (y - 2) >= scroll { 
            window.mv(y - scroll, x);

            if self.num_children() > 0 {
                if self.is_open {
                    window.addstr("[-]"); 
                } else { 
                    window.addstr("[+]"); 
                }
            }

            window.addstr(&self.text);
            if self.file_location.as_str() != "" {
                window.addstr(": ");
                window.addstr(get_file_name(&self.file_location).as_str());
            }
        }

        if self.num_children() > 0 && self.is_open {
            self.render_children(window, x, y, scroll);
        }
    }

    pub fn render_children(&self, window: &Window, x: i32, y: i32, scroll: i32) {
        if self.num_children() > 0 {
            let mut y_off = 1;
            for (i, child) in self.get_children().iter().enumerate() {
                if i > 0 {
                    y_off += self.get_children()[i - 1].flatten_children().len() as i32;
                }

                child.render_entry(window, x + self.child_indent_depth, y + (i as i32) + y_off, scroll);
            }
        }
    }

    pub fn toggle_open(&mut self) {
        self.is_open = !self.is_open; 
    }

    pub fn set_is_open(&mut self, is_open: bool) {
        self.is_open = is_open; 
    }

    pub fn open_file(&self) {
        let mut child = Command::new("vim")
                        .arg(self.file_location.as_str())
                        .spawn()
                        .expect("failed to execute process");

        child.wait();

        curs_set(1);
        curs_set(0);

    }

    pub fn get_id(&self) -> u32 {
        self.id 
    }

    pub fn flatten_children<'a>(&self) -> Vec<&Entry> {
        let mut flattened_children: Vec<&Entry> = Vec::new();

        if self.is_open && !self.is_deleted {
            for child in self.children.iter() {
                if !child.is_deleted {
                    flattened_children.push(child); 
                    flattened_children.append(child.flatten_children().borrow_mut());
                }
            }
        }

        flattened_children
    }

    pub fn get_children(&self) -> Vec<&Entry> {
        let mut children: Vec<&Entry> = Vec::new();

        for child in self.children.iter() {
            if !child.is_deleted {
                children.push(child); 
            } 
        }

        children
    }

    pub fn get_children_mut(&mut self) -> Vec<&mut Entry> {
        let mut children: Vec<&mut Entry> = Vec::new();

        for child in self.children.iter_mut() {
            if !child.is_deleted {
                children.push(child); 
            } 
        }

        children
    }

    pub fn num_children(&self) -> i32{
        let mut n = 0;

        for child in self.children.iter() {
            if !child.is_deleted {
                n += 1; 
            } 
        }

        n
    }

    pub fn get_nth_child(&self, mut n: i32) -> Option<&Entry> {
        if n < 0 {
            return Some(self); 
        } 

        for child in self.get_children().iter() {
            if (n - (child.flatten_children().len() as i32) - 1) < 0 {
                return child.get_nth_child(n - 1); 
            } else {
                n -= (child.flatten_children().len() as i32) + 1;
            }
        }

        None
    }

    pub fn get_child_by_id(&self, id: u32) -> Option<&Entry> {
        if self.id == id {
            return Some(self); 
        }

        for child in self.children.iter() {
            match child.get_child_by_id(id) {
                Some(entry) => return Some(entry),
                None => continue
            } 
        }

        None
    }

    pub fn get_mut_child_by_id(&mut self, id: u32) -> Option<&mut Entry> {
        if self.id == id {
            return Some(self); 
        }

        for child in self.children.iter_mut() {
            match child.get_mut_child_by_id(id) {
                Some(entry) => return Some(entry),
                None => continue
            };
        }

        None
    }

    pub fn get_parent_by_id(&self, id: u32) -> Option<&Entry> {
        for child in self.children.iter() {
            if child.get_id() == id {
                return Some(self); 
            } else {
                match child.get_parent_by_id(id) {
                    Some(parent) => return Some(parent),
                    None => continue
                } 
            }
        }

        None
    }

    pub fn delete_child_by_id(&mut self, id: u32) {
        let mut remove = self.children.len();

        for (i, child) in self.children.iter_mut().enumerate() {
            if child.get_id() == id {
                remove = i;
                break;
            } else {
                child.delete_child_by_id(id); 
            } 
        }

        if remove < self.children.len() {
            self.children.remove(remove); 
        }
    }

    pub fn build_entry_from_json(json_content: &json::JsonValue) -> Entry {
        let text = json_content["text"].as_str().unwrap();
        let mut file_location = "";
        let id = json_content["id"].as_u32().unwrap();
        let sort_type = get_sort_type_from_int(json_content["sort_type"].as_i8().unwrap());
        let sort_descending = try_unwrap(json_content["sort_descending"].as_bool(), false);
        let time_created = json_content["time_created"].as_i64().unwrap();
        let is_category = json_content["is_category"].as_bool().unwrap();

        if json_content["file_location"] != json::Null {
            file_location = json_content["file_location"].as_str().unwrap(); 
        }

        let mut entry = EntryBuilder::new(id).set_text(text).set_is_category(is_category).set_full_file_location(String::from(file_location)).set_sort_type(sort_type).set_sort_descending(sort_descending).set_time_created(time_created).build();

        for child in json_content["children"].members() {
            let child_entry = Entry::build_entry_from_json(child); 
            entry.add_child(child_entry);
        } 

        entry
    }

    pub fn get_as_json(&self) -> json::JsonValue {
        let mut json_children = Vec::new();

        for child in self.get_children().iter() {
                json_children.push(child.get_as_json()); 
        }
      
        if self.file_location.as_str() == "" {
            object! {
                "id" => self.id,
                "is_category" => self.is_category,
                "text" => self.text.as_str(),
                "children" => json_children,
                "file_location" => json::Null,
                "time_created" => self.time_created.timestamp(),
                "sort_type" => get_int_from_sort_type(self.sort_type.clone()),
                "sort_descending" => self.sort_descending,
            }         
        } else {
            object! {
                "id" => self.id,
                "is_category" => self.is_category,
                "text" => self.text.as_str(),
                "children" => json_children,
                "file_location" => self.file_location.as_str(),
                "time_created" => self.time_created.timestamp(),
                "sort_type" => get_int_from_sort_type(self.sort_type.clone()),
                "sort_descending" => self.sort_descending,
            } 
        }
    }

    pub fn sort_children(&mut self) {
        match self.sort_type {
            SortType::NAME => self.children.sort_by(|a, b| a.text.to_lowercase().cmp(&b.text.to_lowercase())), 
            SortType::FILE => self.children.sort_by(|a, b| a.file_location.to_lowercase().cmp(&b.file_location.to_lowercase())),
            SortType::TIME => self.children.sort_by(|a, b| a.time_created.cmp(&b.time_created)),
        }

        if self.sort_descending {
            self.children.reverse(); 
        }
    }

    pub fn set_sort_type(&mut self, sort_type: SortType) {
        self.sort_type = sort_type; 
    }

    pub fn set_sort_descending(&mut self, sort_descending: bool) {
        self.sort_descending = sort_descending;
    }

    pub fn get_sort_type(&self) -> SortType {
        self.sort_type.clone() 
    }

    pub fn get_sort_descending(&self) -> bool {
        self.sort_descending 
    }

    pub fn activate(&mut self) {
        if !self.is_category && self.file_location.as_str() != "" {
            self.open_file();
        } else {
            self.toggle_open() 
        } 
    }
    
    pub fn delete(&mut self) {
        self.is_deleted = true; 
    }

    pub fn undo_delete(&mut self) {
        self.is_deleted = false; 
    }

    pub fn change_name(&mut self, new_name: &str) {
        self.text = String::from(new_name);  
    }
}

fn get_sort_type_from_int(val: i8) -> SortType {
    match val {
        0 => SortType::NAME,
        1 => SortType::FILE,
        2 => SortType::TIME,
        _ => SortType::NAME,
    }
}

fn get_int_from_sort_type(val: SortType) -> i8 {
    match val {
        SortType::NAME => 0,
        SortType::FILE => 1,
        SortType::TIME => 2,
    }
}

fn get_file_name(file_path: &String) -> String {
    let index = &file_path[0..file_path.len() - 1].rfind('/').unwrap();

    String::from(&file_path[index + 1..file_path.len()])
}

pub fn build_full_file_path(mut file_path: String, nm: &note_manager::NoteManager, category_id: u32, is_category: bool) -> String {
    if is_category && &file_path[file_path.len() - 1..file_path.len()] != "/" {
        file_path = format!("{}/", file_path);
    } else if !is_category && &file_path[file_path.len() - 1..file_path.len()] == "/" {
        file_path = String::from(&file_path[0..file_path.len() - 1]);     
    }

    if &file_path[0..1] == "/" {
        file_path 
    } else {
        let mut new_file_path = nm.root.get_child_by_id(category_id).unwrap().file_location.clone();
        new_file_path.push_str(file_path.as_str());

        new_file_path
    }
}

impl Clone for Entry {
    fn clone(&self) -> Entry {
        let mut clone_children: Vec<Entry> = Vec::new();

        for child in self.children.iter() {
            clone_children.push(child.clone()); 
        }

        Entry {
            id: self.id,
            is_category: self.is_category,
            text: self.text.clone(),
            children: clone_children,
            child_indent_depth: self.child_indent_depth,
            is_open: self.is_open,
            file_location: self.file_location.clone(),
            is_deleted: self.is_deleted,
            time_created: self.time_created.clone(),
            sort_type: self.sort_type.clone(),
            sort_descending: self.sort_descending,
        }
    }    
}

pub fn try_unwrap<T>(option: Option<T>, default: T) -> T {
    match option {
        Some(val) => val,
        None => default,
    }
}
