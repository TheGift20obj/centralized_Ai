/*use super::types::ChatMessage;
use ic_cdk::storage;
use once_cell::sync::OnceCell;

static HISTORY: OnceCell<Vec<ChatMessage>> = OnceCell::new();

/// Inicjalizacja pamięci
pub fn init() {
    HISTORY.set(Vec::new()).ok();
}

pub fn pre_upgrade() {
    let h = HISTORY.get().expect("state not initialized");
    storage::stable_save((h,)).expect("failed to save stable state");
}

pub fn post_upgrade() {
    let (h,): (Vec<ChatMessage>,) = storage::stable_restore().expect("failed to restore stable state");
    HISTORY.set(h).ok();
}

/// Zapis nowego wpisu
pub fn save_message(msg: &ChatMessage) {
    let h: &mut std::vec::Vec<ChatMessage> = HISTORY.get().expect("state not initialized");
    h.push(msg.clone());
}

/// Pobranie całej historii
pub fn get_history() -> Vec<ChatMessage> {
    HISTORY.get().expect("state not initialized").clone()
}*/