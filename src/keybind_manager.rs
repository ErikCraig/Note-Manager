use std::collections::HashMap;
use note_manager::*;
use pancurses::*;

#[derive(Clone)]
pub enum KeybindMode {
    DEFAULT,
    MULTIKEY(String),
    MOVE(u32),
    ALL
}

pub struct Keybind {
    pub mode: KeybindMode,
    pub function: Box<FnMut(&mut NoteManager, &KeybindMode) -> Option<KeybindMode>>,
}

pub struct KeybindManager {
    nm: NoteManager,
    mode: KeybindMode,
    keybindings: HashMap<String, Vec<Keybind>>,
}

impl KeybindManager {
    pub fn new(nm: NoteManager) -> KeybindManager {
        KeybindManager {
            nm: nm,
            mode: KeybindMode::DEFAULT,
            keybindings: HashMap::new(),
        }
    }

    pub fn add<F: 'static>(&mut self, keys: &str, mode: KeybindMode, function: F) where
        F: FnMut(&mut NoteManager, &KeybindMode) -> Option<KeybindMode>
    {
        let keybind = Keybind::new(mode, function);
        if self.keybindings.get_mut(&String::from(keys)).is_none() {
            self.keybindings.insert(String::from(keys), vec![keybind]); 
        } else {
            self.keybindings.get_mut(&String::from(keys)).unwrap().push(keybind);
        }
    }

    pub fn handle_input(&mut self, key: char) {
        let mut keybind = String::new();

        match self.mode {
            KeybindMode::MULTIKEY(ref prev_keys) => {
                keybind = prev_keys.clone();
                keybind.push(key);
            },
            _ => {
                keybind.push(key); 
            },
        }

        let mut new_mode = KeybindMode::DEFAULT;

        match self.keybindings.get_mut(&keybind) {
            Some(keybinding) => {  
                for k in keybinding {
                    if self.mode == k.mode { 
                        match (k.function)(&mut self.nm, &self.mode) {
                            Some(m) => new_mode = m,
                            None => new_mode = self.mode.clone(),
                        }
                    }
                }
            },
            None => (),
        }

        for hash_map_key in self.keybindings.keys() {
            if is_partial_keybind(&keybind, hash_map_key) {
                new_mode = KeybindMode::MULTIKEY(keybind); 
                break;
            }
        }

        self.mode = new_mode;
    }

    pub fn begin(&mut self) {
        self.nm.render();
        while self.nm.running {
            match self.nm.window.getch() {
                Some(Input::Character(input)) => {
                    self.handle_input(input);
                },
                _ => (),
            }
        }
    }

    pub fn end(&mut self) {
        self.nm.end(); 
    }
}

impl Keybind {
    pub fn new<F: 'static>(mode: KeybindMode, function: F) -> Keybind where
        F: FnMut(&mut NoteManager, &KeybindMode) -> Option<KeybindMode>
    {
        Keybind {
            mode: mode,
            function: Box::new(function),
        } 
    }
}

impl PartialEq for KeybindMode {
    fn eq(&self, other: &KeybindMode) -> bool {
        match (self, other) {
            (KeybindMode::DEFAULT, KeybindMode::DEFAULT) => true,
            (KeybindMode::MULTIKEY(_), KeybindMode::MULTIKEY(_)) => true,
            (KeybindMode::MOVE(_), KeybindMode::MOVE(_)) => true,
            (_, KeybindMode::ALL) => true,
            (KeybindMode::ALL, _) => true,
            _ => false,
        } 
    }
}

fn is_partial_keybind(keys1: &String, keys2: &String) -> bool {
    if keys1 == keys2 {
        return false; 
    }

    keys2.starts_with(keys1.as_str())
} 
