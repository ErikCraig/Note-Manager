use note_manager;
use notes;

use std::collections::VecDeque;

pub struct ActionList {
    undo_list: Vec<Box<Action>>,
    redo_list: VecDeque<Box<Action>>,
}

pub trait Action {
    fn activate(&self, nm: &mut note_manager::NoteManager);
    fn undo(&self, nm: &mut note_manager::NoteManager);
}

pub struct DeleteAction {
    deleted_id: u32,
}

pub struct AddAction {
    added_id: u32,
    entry_id: u32,
    child: notes::Entry,
}

pub struct RenameAction {
    renamed_id: u32,
    new_name: String,
    old_name: String,
}

pub struct ChangeFileAction {
    changed_id: u32,
    new_location: String,
    old_location: String,
}

pub struct SortAction {
    id: u32,
    new_sort_type: notes::SortType,
    old_sort_type: notes::SortType,
}

pub struct SortDirectionAction {
    id: u32, 
    new_sort_descending: bool,
    old_sort_descending: bool,
}

pub struct MoveAction {
    id: u32,
    old_category_id: u32,
    new_category_id: u32,
}

impl ActionList {
    pub fn new() -> ActionList {
        ActionList {
            undo_list: Vec::new(),
            redo_list: VecDeque::new(),
        } 
    }

    pub fn add(&mut self, action: Box<Action>) {
        self.undo_list.push(action); 
        self.redo_list.clear();
    }

    pub fn add_redo(&mut self, action: Box<Action>) {
        self.redo_list.push_back(action); 
    }

    pub fn add_undo(&mut self, action: Box<Action>) {
        self.undo_list.push(action); 
    }

    //Make these options to allow for error handling
    pub fn get_undo(&mut self) -> Option<Box<Action>> {
        return self.undo_list.pop();
    }

    pub fn get_redo(&mut self) -> Option<Box<Action>> {
        return self.redo_list.pop_front(); 
    } 
}

impl DeleteAction {
    pub fn new(id: u32) -> DeleteAction {
        DeleteAction {
            deleted_id: id,
        } 
    }
}

impl Action for DeleteAction {
    fn activate(&self, nm: &mut note_manager::NoteManager) {
        nm.root.get_mut_child_by_id(self.deleted_id).unwrap().delete();

        if nm.cursor >= nm.root.flatten_children().len() as i32 && nm.cursor > 0 {
            nm.cursor = (nm.root.flatten_children().len() as i32) - 1;
        }

        nm.render();
    }

    fn undo(&self, nm: &mut note_manager::NoteManager) {
        nm.root.get_mut_child_by_id(self.deleted_id).unwrap().undo_delete();
        nm.render();
    }
}

impl AddAction {
    pub fn new(entry_id: u32, child: notes::Entry) -> AddAction {
        AddAction {
            added_id: child.get_id(),
            entry_id: entry_id,
            child: child,
        } 
    }
}

impl Action for AddAction {
    fn activate(&self, nm: &mut note_manager::NoteManager) {
        if nm.root.get_child_by_id(self.child.get_id()).is_none() {
            nm.root.get_mut_child_by_id(self.entry_id).unwrap().add_child(self.child.clone());
            nm.next_id += 1;
        } else {
            nm.root.get_mut_child_by_id(self.child.get_id()).unwrap().undo_delete(); 
        }
        
        nm.render();
    } 

    fn undo(&self, nm: &mut note_manager::NoteManager) {
        nm.root.get_mut_child_by_id(self.added_id).unwrap().delete(); 
        nm.render();
    }
}

impl RenameAction {
    pub fn new(entry: &notes::Entry, new_name: &str) -> RenameAction {
        RenameAction {
            renamed_id: entry.get_id(),
            new_name: String::from(new_name),
            old_name: entry.text.clone(),
        } 
    }
}

impl Action for RenameAction {
    fn activate(&self, nm: &mut note_manager::NoteManager) {
        nm.root.get_mut_child_by_id(self.renamed_id).unwrap().change_name(self.new_name.as_str()); 
        nm.render();
    }

    fn undo(&self, nm: &mut note_manager::NoteManager) {
        nm.root.get_mut_child_by_id(self.renamed_id).unwrap().change_name(self.old_name.as_str()); 
        nm.render();
    }
}

impl ChangeFileAction {
    pub fn new(entry: &notes::Entry, new_location: &str) -> ChangeFileAction {
        ChangeFileAction {
            changed_id: entry.get_id(),
            new_location: String::from(new_location),
            old_location: entry.file_location.clone(),
        } 
    }
}

impl Action for ChangeFileAction {
    fn activate(&self, nm: &mut note_manager::NoteManager) {
        let new_file = notes::build_full_file_path(self.new_location.clone(), nm, nm.root.get_parent_by_id(self.changed_id).unwrap().get_id(), nm.root.get_child_by_id(self.changed_id).unwrap().can_add_child());
        nm.root.get_mut_child_by_id(self.changed_id).unwrap().file_location = new_file;
        nm.render();
    }

    fn undo(&self, nm: &mut note_manager::NoteManager) {
        nm.root.get_mut_child_by_id(self.changed_id).unwrap().file_location = self.old_location.clone();
        nm.render();
    }
}

impl SortAction {
    pub fn new(entry: &notes::Entry, new_sort_type: notes::SortType, old_sort_type: notes::SortType) -> SortAction {
        SortAction {
            id: entry.get_id(), 
            new_sort_type: new_sort_type,
            old_sort_type: old_sort_type,
        }  
    }
}

impl Action for SortAction {
    fn activate(&self, nm: &mut note_manager::NoteManager) {
        {
            let category = nm.root.get_mut_child_by_id(self.id).unwrap();
            category.set_sort_type(self.new_sort_type.clone());
            category.sort_children();
        }
        
        nm.render();
    }

    fn undo(&self, nm: &mut note_manager::NoteManager) {
        {
            let category = nm.root.get_mut_child_by_id(self.id).unwrap();
            category.set_sort_type(self.old_sort_type.clone());
        }
        nm.render(); 
    }
}

impl SortDirectionAction {
    pub fn new(entry: &notes::Entry, new_sort_descending: bool, old_sort_descending: bool) -> SortDirectionAction {
        SortDirectionAction {
            id: entry.get_id(),
            new_sort_descending: new_sort_descending,
            old_sort_descending: old_sort_descending,
        } 
    }
}

impl Action for SortDirectionAction {
    fn activate(&self, nm: &mut note_manager::NoteManager) {
        {
            let category = nm.root.get_mut_child_by_id(self.id).unwrap();
            category.set_sort_descending(self.new_sort_descending);
            category.sort_children();
        } 

        nm.render();
    }

    fn undo(&self, nm: &mut note_manager::NoteManager) {
        {
            let category = nm.root.get_mut_child_by_id(self.id).unwrap();
            category.set_sort_descending(self.old_sort_descending);
        } 

        nm.render();
    }
}

impl MoveAction {
    pub fn new(entry_id: u32, old_category_id: u32, new_category_id: u32) -> MoveAction {
        MoveAction {
            id: entry_id,
            old_category_id: old_category_id,
            new_category_id: new_category_id,
        } 
    }
}

impl Action for MoveAction {
    fn activate(&self, nm: &mut note_manager::NoteManager) {
        if self.id != self.new_category_id {
            let entry = nm.root.get_child_by_id(self.id).unwrap().clone();

            if nm.root.get_child_by_id(self.new_category_id).unwrap().can_add_child() {
                nm.root.delete_child_by_id(self.id);
                nm.root.get_mut_child_by_id(self.new_category_id).unwrap().add_child(entry);
            }
        }

        if nm.cursor >= (nm.root.flatten_children().len() as i32) {
            nm.cursor = (nm.root.flatten_children().len() - 1) as i32; 
        }

        nm.render();
    }

    fn undo(&self, nm: &mut note_manager::NoteManager) {
        if self.id != self.new_category_id {
            let entry = nm.root.get_child_by_id(self.id).unwrap().clone();

            if nm.root.get_child_by_id(self.old_category_id).unwrap().can_add_child() {
                nm.root.delete_child_by_id(self.id);
                nm.root.get_mut_child_by_id(self.old_category_id).unwrap().add_child(entry);
            }
        }

        nm.render();
    }
} 
