use keybind_manager;
use actions::*;
use notes;
use note_manager;

pub fn init_keybindings(kbm: &mut keybind_manager::KeybindManager) {
    kbm.add("d", keybind_manager::KeybindMode::DEFAULT, delete_entry);
    kbm.add("an", keybind_manager::KeybindMode::MULTIKEY(String::from("")), add_note);
    kbm.add("j", keybind_manager::KeybindMode::ALL, cursor_down);
    kbm.add("k", keybind_manager::KeybindMode::ALL, cursor_up);
    kbm.add("u", keybind_manager::KeybindMode::DEFAULT, undo);
    kbm.add("al", keybind_manager::KeybindMode::MULTIKEY(String::from("")), redo);
    kbm.add("\n", keybind_manager::KeybindMode::DEFAULT, select);
    kbm.add("q", keybind_manager::KeybindMode::DEFAULT, close);
    kbm.add("m", keybind_manager::KeybindMode::DEFAULT, move_start);
    kbm.add("\n", keybind_manager::KeybindMode::MOVE(0), move_complete);
    kbm.add("ac", keybind_manager::KeybindMode::MULTIKEY(String::from("")), add_category);
    kbm.add("ar", keybind_manager::KeybindMode::MULTIKEY(String::from("")), add_root_category);
    kbm.add("cn", keybind_manager::KeybindMode::MULTIKEY(String::from("")), change_name);
    kbm.add("cf", keybind_manager::KeybindMode::MULTIKEY(String::from("")), change_file);
    kbm.add("sn", keybind_manager::KeybindMode::MULTIKEY(String::from("")), |nm, mode| {
        sort_category(nm, mode, notes::SortType::NAME) 
    });
    kbm.add("sf", keybind_manager::KeybindMode::MULTIKEY(String::from("")), |nm, mode| {
        sort_category(nm, mode, notes::SortType::FILE) 
    });
    kbm.add("st", keybind_manager::KeybindMode::MULTIKEY(String::from("")), |nm, mode| {
        sort_category(nm, mode, notes::SortType::TIME) 
    });
    kbm.add("sd", keybind_manager::KeybindMode::MULTIKEY(String::from("")), |nm, mode| {
        sort_direction(nm, mode, true)  
    });
    kbm.add("sa", keybind_manager::KeybindMode::MULTIKEY(String::from("")), |nm, mode| {
        sort_direction(nm, mode, false) 
    });
}

fn delete_entry(nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
    let delete = DeleteAction::new(nm.root.get_nth_child(nm.cursor).unwrap().get_id());
    delete.activate(nm);

    nm.actions.add(Box::new(delete));

    Some(keybind_manager::KeybindMode::DEFAULT)
}

fn cursor_up(nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
    nm.move_cursor(-1);
    None
}

fn cursor_down(nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
    nm.move_cursor(1);
    None
}

fn undo(nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
    match nm.actions.get_undo() {
        Some(action) => {
            action.undo(nm);
            nm.actions.add_redo(action);
        }, 
        None => (),
    }   
    Some(keybind_manager::KeybindMode::DEFAULT)
}

fn redo(nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
     match nm.actions.get_redo() {
        Some(action) => {
            action.activate(nm);
            nm.actions.add_undo(action);
        },
        None => ()
    }
    Some(keybind_manager::KeybindMode::DEFAULT)
}

fn select(nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
    match nm.selected {
        Some(selected) => {
            let move_action = MoveAction::new(selected, nm.root.get_parent_by_id(selected).unwrap().get_id(), nm.root.get_nth_child(nm.cursor).unwrap().get_id());
            move_action.activate(nm);

            nm.actions.add(Box::new(move_action));

            nm.selected = None;
        },
        None => {
            let id = nm.root.get_nth_child(nm.cursor).unwrap().get_id();
            nm.root.get_mut_child_by_id(id).unwrap().activate();
            nm.render();
        }
    }
    Some(keybind_manager::KeybindMode::DEFAULT)
}

fn close(nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
    nm.end(); 
    Some(keybind_manager::KeybindMode::DEFAULT)
}

fn move_start(nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
    match nm.root.get_nth_child(nm.cursor) {
        Some(child) => Some(keybind_manager::KeybindMode::MOVE(child.get_id())),
        None => None,
    }
}

fn move_complete(nm: &mut note_manager::NoteManager, mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
    match mode {
        keybind_manager::KeybindMode::MOVE(id) => {
            let move_action = MoveAction::new(*id, nm.root.get_parent_by_id(*id).unwrap().get_id(), nm.root.get_nth_child(nm.cursor).unwrap().get_id());
            
            move_action.activate(nm);
            nm.actions.add(Box::new(move_action));
        },
        _ => (),
    }

    Some(keybind_manager::KeybindMode::DEFAULT)
}

fn add_note(nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
    if nm.root.get_nth_child(nm.cursor).unwrap().can_add_child() {
        match nm.get_text_input("Input new note name") {
            Some(name) => {
                match nm.get_text_input("Input path to file") {
                    Some(file_location) => {
                        let new_child = notes::EntryBuilder::new(nm.next_id).set_text(name.as_str()).set_file_location(file_location, nm, nm.root.get_nth_child(nm.cursor).unwrap().get_id()).build();
                        let add = AddAction::new(nm.root.get_nth_child(nm.cursor).unwrap().get_id(), new_child);
                        add.activate(nm);

                        nm.actions.add(Box::new(add));
                    },
                    None => () 
                }
            },
            None => () 
        }
    }   

    Some(keybind_manager::KeybindMode::DEFAULT)
}

fn add_category(nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
    if nm.root.get_children().len() > 0 {
        match nm.get_text_input("Input new category name") {
            Some(name) => {
                match nm.get_text_input("Input path to directory") {
                    Some(file_location) => {
                        let category_id = nm.root.get_nth_child(nm.cursor).unwrap().get_id();

                        let new_child = notes::EntryBuilder::new(nm.next_id).set_text(name.as_str()).set_is_category(true).set_file_location(file_location, nm, category_id).build();

                        let add = AddAction::new(category_id, new_child);
                        add.activate(nm);

                        nm.actions.add(Box::new(add));
                    },
                    None => (),
                }
            },
            None => (),
        } 
    } else {
        nm.display_message("No top level categories exist. Press \"ar\" to add one"); 
    }   

    Some(keybind_manager::KeybindMode::DEFAULT)
}

fn add_root_category(nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
    match nm.get_text_input("Input new category name") {
        Some(name) => {
            match nm.get_text_input("Input path to directory") {
                Some(file_location) => {
                    let new_child = notes::EntryBuilder::new(nm.next_id).set_text(name.as_str()).set_is_category(true).set_file_location(file_location, nm, nm.root.get_id()).build();
                    let add = AddAction::new(nm.root.get_id(), new_child);
                    add.activate(nm);

                    nm.actions.add(Box::new(add));
                },
                None => (),
            }
        },
        None => () 
    }   

    Some(keybind_manager::KeybindMode::DEFAULT)
} 

fn change_name(nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
    match nm.get_text_input("Input new name") {
        Some (name) => {
            let rename = RenameAction::new(nm.root.get_nth_child(nm.cursor).unwrap(), name.as_str());
            rename.activate(nm);

            nm.actions.add(Box::new(rename));
        },
        None => ()
    }

    Some(keybind_manager::KeybindMode::DEFAULT)
}

fn change_file (nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode) -> Option<keybind_manager::KeybindMode> {
    match nm.get_text_input("Input new file location") {
        Some(name) => {
            let change_file = ChangeFileAction::new(nm.root.get_nth_child(nm.cursor).unwrap(), name.as_str());
            change_file.activate(nm);

            nm.actions.add(Box::new(change_file));
        },
        None => ()
    }   

    Some(keybind_manager::KeybindMode::DEFAULT)
}

fn sort_category (nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode, sort_type: notes::SortType) -> Option<keybind_manager::KeybindMode> {
    let sort_action = SortAction::new(nm.root.get_nth_child(nm.cursor).unwrap(), sort_type, nm.root.get_nth_child(nm.cursor).unwrap().get_sort_type());
    sort_action.activate(nm);

    nm.actions.add(Box::new(sort_action));

    Some(keybind_manager::KeybindMode::DEFAULT)
}

fn sort_direction (nm: &mut note_manager::NoteManager, _mode: &keybind_manager::KeybindMode, sort_descending: bool) -> Option<keybind_manager::KeybindMode> {
    let sort_direction_action = SortDirectionAction::new(nm.root.get_nth_child(nm.cursor).unwrap(), sort_descending, nm.root.get_nth_child(nm.cursor).unwrap().get_sort_descending());
    sort_direction_action.activate(nm);

    nm.actions.add(Box::new(sort_direction_action));

    Some(keybind_manager::KeybindMode::DEFAULT)
}
