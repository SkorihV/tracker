//! Справочники (пользователи, теги, клиенты, статусы): одна точка входа
//! `Store::edit_dictionary` вместо отдельных методов на каждую операцию.

use serde::Deserialize;

use crate::models::{Entity, StatusDef};
use crate::store::{shift_vec, Store};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DictKind {
    User,
    Tag,
    Client,
    Status,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum DictAction {
    Add {
        name: String,
        #[serde(default)]
        color: Option<String>,
    },
    Rename {
        old: String,
        name: String,
    },
    Remove {
        #[serde(default)]
        id: Option<u64>,
        #[serde(default)]
        name: Option<String>,
    },
    Move {
        from: usize,
        to: usize,
    },
    Clear,
    Color {
        id: u64,
        color: String,
    },
}

/// Добавить сущность, если имя непустое и ещё не занято. `true` — список изменён.
fn add_entity(list: &mut Vec<Entity>, next_id: &mut u64, name: &str) -> bool {
    let name = name.trim();
    if name.is_empty() || list.iter().any(|e| e.name == name) {
        return false;
    }
    list.push(Entity {
        id: *next_id,
        name: name.to_string(),
    });
    *next_id += 1;
    true
}

fn rename_entity(list: &mut [Entity], old: &str, new: &str) {
    if let Some(e) = list.iter_mut().find(|e| e.name == old) {
        e.name = new.to_string();
    }
}

impl Store {
    // ---- внутренние «ensure»: добавляют в справочник без сохранения файла ----

    pub(crate) fn ensure_user(&mut self, name: &str) {
        let name = name.trim();
        if !name.is_empty() && !self.users.iter().any(|u| u == name) {
            self.users.push(name.to_string());
        }
    }

    pub(crate) fn ensure_tag(&mut self, name: &str) {
        add_entity(&mut self.tags, &mut self.next_tag_id, name);
    }

    pub(crate) fn ensure_client(&mut self, name: &str) {
        add_entity(&mut self.clients, &mut self.next_client_id, name);
    }

    pub(crate) fn ensure_status(&mut self, name: &str) {
        self.push_status(name, "default");
    }

    fn push_status(&mut self, name: &str, color: &str) -> bool {
        let name = name.trim();
        if name.is_empty() || self.statuses.iter().any(|s| s.name == name) {
            return false;
        }
        self.statuses.push(StatusDef {
            id: self.next_status_id,
            name: name.to_string(),
            color: color.trim().to_string(),
        });
        self.next_status_id += 1;
        true
    }

    // ---- публичные операции (с сохранением) ----

    pub fn add_user(&mut self, name: &str) -> Result<(), String> {
        self.edit_dictionary(DictKind::User, add_action(name))
    }

    pub fn add_tag(&mut self, name: &str) -> Result<(), String> {
        self.edit_dictionary(DictKind::Tag, add_action(name))
    }

    pub fn add_client(&mut self, name: &str) -> Result<(), String> {
        self.edit_dictionary(DictKind::Client, add_action(name))
    }

    pub fn add_status(&mut self, name: &str, color: &str) -> Result<(), String> {
        let action = DictAction::Add {
            name: name.to_string(),
            color: Some(color.to_string()),
        };
        self.edit_dictionary(DictKind::Status, action)
    }

    pub fn rename_tag(&mut self, old: &str, new: &str) -> Result<(), String> {
        self.edit_dictionary(DictKind::Tag, rename_action(old, new))
    }

    pub fn rename_status(&mut self, old: &str, new: &str) -> Result<(), String> {
        self.edit_dictionary(DictKind::Status, rename_action(old, new))
    }

    pub fn remove_status(&mut self, id: u64) -> Result<(), String> {
        self.edit_dictionary(
            DictKind::Status,
            DictAction::Remove {
                id: Some(id),
                name: None,
            },
        )
    }

    /// Применить операцию к справочнику и сохранить файл, если что-то изменилось.
    pub fn edit_dictionary(&mut self, kind: DictKind, action: DictAction) -> Result<(), String> {
        let changed = match action {
            DictAction::Add { name, color } => self.dict_add(kind, &name, color.as_deref()),
            DictAction::Rename { old, name } => self.dict_rename(kind, &old, name.trim()),
            DictAction::Remove { id, name } => self.dict_remove(kind, id, name.as_deref()),
            DictAction::Move { from, to } => match kind {
                DictKind::User => shift_vec(&mut self.users, from, to),
                DictKind::Tag => shift_vec(&mut self.tags, from, to),
                DictKind::Client => shift_vec(&mut self.clients, from, to),
                DictKind::Status => shift_vec(&mut self.statuses, from, to),
            },
            DictAction::Clear => self.dict_clear(kind),
            DictAction::Color { id, color } => match self.statuses.iter_mut().find(|s| s.id == id) {
                Some(st) if kind == DictKind::Status => {
                    st.color = color.trim().to_string();
                    true
                }
                _ => false,
            },
        };
        if changed {
            self.save()
        } else {
            Ok(())
        }
    }

    fn dict_add(&mut self, kind: DictKind, name: &str, color: Option<&str>) -> bool {
        match kind {
            DictKind::User => {
                let before = self.users.len();
                self.ensure_user(name);
                self.users.len() != before
            }
            DictKind::Tag => add_entity(&mut self.tags, &mut self.next_tag_id, name),
            DictKind::Client => add_entity(&mut self.clients, &mut self.next_client_id, name),
            DictKind::Status => self.push_status(name, color.unwrap_or("default")),
        }
    }

    fn dict_rename(&mut self, kind: DictKind, old: &str, new: &str) -> bool {
        if new.is_empty() {
            return false;
        }
        match kind {
            DictKind::User => {
                for u in self.users.iter_mut().filter(|u| *u == old) {
                    *u = new.to_string();
                }
                if self.settings.username == old {
                    self.settings.username = new.to_string();
                }
                for t in self.tasks.iter_mut().filter(|t| t.user == old) {
                    t.user = new.to_string();
                }
            }
            DictKind::Tag => {
                rename_entity(&mut self.tags, old, new);
                for tag in self.tasks.iter_mut().flat_map(|t| t.tags.iter_mut()) {
                    if tag == old {
                        *tag = new.to_string();
                    }
                }
            }
            DictKind::Client => {
                rename_entity(&mut self.clients, old, new);
                for t in self.tasks.iter_mut().filter(|t| t.client == old) {
                    t.client = new.to_string();
                }
            }
            DictKind::Status => {
                if let Some(s) = self.statuses.iter_mut().find(|s| s.name == old) {
                    s.name = new.to_string();
                }
                for t in self.tasks.iter_mut().filter(|t| t.custom_status == old) {
                    t.custom_status = new.to_string();
                }
            }
        }
        true
    }

    fn dict_remove(&mut self, kind: DictKind, id: Option<u64>, name: Option<&str>) -> bool {
        match kind {
            DictKind::User => {
                let Some(name) = name else { return false };
                self.users.retain(|u| u != name);
                if self.settings.username == name {
                    self.settings.username.clear();
                }
            }
            DictKind::Tag => self.tags.retain(|e| Some(e.id) != id),
            DictKind::Client => self.clients.retain(|e| Some(e.id) != id),
            DictKind::Status => {
                let removed: Vec<String> = self
                    .statuses
                    .iter()
                    .filter(|s| Some(s.id) == id)
                    .map(|s| s.name.clone())
                    .collect();
                self.statuses.retain(|s| Some(s.id) != id);
                for t in self.tasks.iter_mut().filter(|t| removed.contains(&t.custom_status)) {
                    t.custom_status.clear();
                }
            }
        }
        true
    }

    fn dict_clear(&mut self, kind: DictKind) -> bool {
        match kind {
            DictKind::User => {
                self.users.clear();
                self.settings.username.clear();
            }
            DictKind::Tag => self.tags.clear(),
            DictKind::Client => self.clients.clear(),
            DictKind::Status => {
                self.statuses.clear();
                for t in self.tasks.iter_mut() {
                    t.custom_status.clear();
                }
            }
        }
        true
    }
}

fn add_action(name: &str) -> DictAction {
    DictAction::Add {
        name: name.to_string(),
        color: None,
    }
}

fn rename_action(old: &str, new: &str) -> DictAction {
    DictAction::Rename {
        old: old.to_string(),
        name: new.to_string(),
    }
}
