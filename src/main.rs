extern crate pancurses;
extern crate chrono;
mod notes;
mod note_manager;
mod actions;
mod keybind_manager;
mod keybindings;

use std::env;
use keybindings::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        println!("You need to input a file name");
        return 
    }

    let file = args[1].clone();

    let note_manager = note_manager::NoteManager::new(String::from("Notes"), file);
    let mut keybind_manager = keybind_manager::KeybindManager::new(note_manager);

    init_keybindings(&mut keybind_manager);

    keybind_manager.begin();
    keybind_manager.end();
}
