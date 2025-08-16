use candid::{Principal, CandidType};
use std::collections::HashMap;
use ic_cdk_macros::{update, query};
use serde::{Deserialize, Serialize};
use ic_cdk::api::time;
use ic_llm::Model;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;

type Memory = VirtualMemory<DefaultMemoryImpl>;

const PROMPT_LIMIT: u32 = 50;
const BLOCK_TIME_NANOS: u64 = 12 * 60 * 60 * 1_000_000_000;

#[derive(Clone, CandidType, Deserialize, Serialize)]
struct ChatMeta {
    name: String,
    id: u64,
    msg_len: u32,
}

#[derive(Clone, CandidType, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
    timestamp: u64,
}

#[derive(Clone, CandidType, Deserialize)]
struct ChatInfo {
    messages: Vec<ChatMessage>,
}

type ChatId = String;

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static USER_NAMES_STABLE: RefCell<StableBTreeMap<Principal, String, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    static USER_PROMPTS_STABLE: RefCell<StableBTreeMap<Principal, (u32, Option<u64>), Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );

    static USER_CHATS_STABLE: RefCell<StableBTreeMap<(Principal, u64), (String, u32), Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    static CHAT_MESSAGES_STABLE: RefCell<StableBTreeMap<((Principal, u64), u32), (String, String, u64), Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );
}

fn inc_user_prompt_stable(user: Principal) -> bool {
    let now = time();

    USER_PROMPTS_STABLE.with(|map_cell| {
        let mut map = map_cell.borrow_mut();

        // pobierz aktualną wartość lub ustaw domyślną
        let mut count = 0;
        let mut blocked_since: Option<u64> = None;

        if let Some((c, b)) = map.get(&user) {
            count = c.clone();
            blocked_since = b.clone();
        }

        // logika blokady
        if let Some(block_time) = blocked_since {
            if now - block_time >= BLOCK_TIME_NANOS {
                // reset
                map.insert(user, (1, None));
                true
            } else {
                false
            }
        } else {
            count += 1;
            if count >= PROMPT_LIMIT {
                blocked_since = Some(now);
            }
            map.insert(user, (count, blocked_since));
            true
        }
    })
}

fn set_name_stable(principal: Principal, value: String) {
    USER_NAMES_STABLE.with(|map| map.borrow_mut().insert(principal, value));
}

fn get_name_stable(principal: Principal) -> Option<String> {
    USER_NAMES_STABLE.with(|map| map.borrow().get(&principal))
}

fn get_chats_for_user(user: Principal) -> Vec<ChatMeta> {
    USER_CHATS_STABLE.with(|map_ref| {
        let map = map_ref.borrow();
        map.iter()
            .filter_map(|entry| {
                let (p, index) = entry.key();
                let (name, msg_count) = entry.value();
                if *p == user {
                    Some(ChatMeta {name, id: *index, msg_len: msg_count })
                } else {
                    None
                }
            })
            .collect()
    })
}

fn get_msgs_for_user(user: Principal, chat_id: u64, msg_count: u32) -> ChatInfo {
    CHAT_MESSAGES_STABLE.with(|map_ref| {
        let map = map_ref.borrow();
        let mut info = ChatInfo { messages: Vec::new() };

        for i in 0..msg_count {
            let key = ((user, chat_id), i);
            if let Some((role, content, timestamp)) = map.get(&key) {
                info.messages.push(ChatMessage { role, content, timestamp });
            }
        }

        info
    })
}

fn create_new_chat_stable(user: Principal, chat_id: u64, name: String) {
    USER_CHATS_STABLE.with(|map| {
        map.borrow_mut().insert((user, chat_id), (name, 0));
    });
}

fn delete_chat_stable(user: Principal, chat_id: u64) -> bool {
    let removed = USER_CHATS_STABLE.with(|map| map.borrow_mut().remove(&(user, chat_id)));
    if removed.is_some() {
        CHAT_MESSAGES_STABLE.with(|map| {
            let mut map = map.borrow_mut();
            let keys_to_remove: Vec<_> = map
                .iter()
                .filter_map(|entry| {
                    let ((p, c), idx) = entry.key();
                    if *p == user && *c == chat_id {
                        Some(((user, chat_id), *idx))
                    } else {
                        None
                    }
                })
                .collect();

            for key in keys_to_remove {
                map.remove(&key);
            }
        });
        true
    } else {
        false
    }
}

fn rename_chat_stable(user: Principal, chat_id: u64, new_name: String) -> bool {
    USER_CHATS_STABLE.with(|map| {
        let mut map = map.borrow_mut();
        if let Some((_old_name, msg_count)) = map.get(&(user, chat_id)) {
            map.insert((user, chat_id), (new_name, msg_count));
            true
        } else {
            false
        }
    })
}

fn add_chat_message_stable(user: Principal, chat_id: u64, content: String, role: String) -> bool {
    let timestamp = time();
    USER_CHATS_STABLE.with(|chat_map| {
        let mut chat_map = chat_map.borrow_mut();
        if let Some((name, msg_count)) = chat_map.get(&(user, chat_id)) {
            let new_index = msg_count;
            CHAT_MESSAGES_STABLE.with(|msg_map| {
                msg_map
                    .borrow_mut()
                    .insert(((user, chat_id), new_index), (role, content, timestamp));
            });
            chat_map.insert((user, chat_id), (name.clone(), new_index + 1));
            true
        } else {
            false
        }
    })
}

thread_local! {
    static USER_CHATS: std::cell::RefCell<HashMap<Principal, HashMap<ChatId, ChatInfo>>> = std::cell::RefCell::new(HashMap::new());
    static USER_NAMES: std::cell::RefCell<HashMap<Principal, String>> = std::cell::RefCell::new(HashMap::new());
    static USER_PROMPTS: std::cell::RefCell<HashMap<Principal, (u32, Option<u64>)>> = std::cell::RefCell::new(HashMap::new());
}
#[derive(Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}


#[update]
async fn chat(prompt: String) -> String {
    ic_llm::prompt(Model::Llama3_1_8B, prompt).await
}

#[update]
fn create_new_chat(user: Principal, uid: u64, name: String) {
    create_new_chat_stable(user, uid, name);
}

#[update]
fn add_chat_message(user: Principal, chat_id: u64, content: String, role: String) {
    let _ = add_chat_message_stable(user, chat_id, content, role);
}

#[query]
fn get_chat_history(user: Principal, chat_id: u64, msg_len: u32) -> ChatInfo {
    get_msgs_for_user(user, chat_id, msg_len)
}

#[update]
fn delete_chat(user: Principal, chat_id: u64) -> bool {
    delete_chat_stable(user, chat_id)
}

#[update]
fn rename_chat(user: Principal, chat_id: u64, new_name: String) -> bool {
    rename_chat_stable(user, chat_id, new_name)
}

#[query]
fn list_chats(user: Principal) -> Vec<ChatMeta> {
    get_chats_for_user(user)
}

#[update]
fn set_user_name(user: Principal, name: String) {
    set_name_stable(user, name)
}

#[query]
fn get_user_name(user: Principal) -> String {
    get_name_stable(user).unwrap_or_else(|| "anonimus".to_string())
}

#[update]
pub fn try_increment_user_prompt(user: Principal) -> bool {
    inc_user_prompt_stable(user)
}