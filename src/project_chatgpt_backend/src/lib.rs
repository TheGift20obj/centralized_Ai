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
    id: ChatId,
    name: String,
}

#[derive(Clone, CandidType, Deserialize)]
struct ChatMessage {
    question: String,
    answer: String,
}

#[derive(Clone, CandidType, Deserialize)]
struct ChatInfo {
    name: String,
    messages: Vec<ChatMessage>,
}

type ChatId = String;

#[derive(CandidType, Deserialize, Serialize, Clone)]
struct Message_Stable {
    sender: Principal,
    text: String,
    timestamp: u64,
}

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

    static USER_CHATS_STABLE: RefCell<StableBTreeMap<(Principal, u32), (String, u32), Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    static CHAT_MESSAGES_STABLE: RefCell<StableBTreeMap<((Principal, u32), u32), (String, u64), Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );
}

fn set_name_stable(principal: Principal, value: String) {
    USER_NAMES_STABLE.with(|map| map.borrow_mut().insert(principal, value));
}

fn get_name_stable(principal: Principal) -> Option<String> {
    USER_NAMES_STABLE.with(|map| map.borrow().get(&principal))
}

fn get_chats_for_user(user: Principal) -> Vec<(String, u32)> {
    USER_CHATS_STABLE.with(|map_ref| {
        let map = map_ref.borrow();
        map.iter()
            .filter_map(|entry| {
                let (p, _index) = entry.key();
                let (name, msg_count) = entry.value();
                if *p == user {
                    Some((name.clone(), msg_count.clone()))
                } else {
                    None
                }
            })
            .collect()
    })
}

fn get_msgs_for_user(user: Principal, chat_id: u32, msg_count: u32) -> Vec<(String, u64)> {
    CHAT_MESSAGES_STABLE.with(|map_ref| {
        let map = map_ref.borrow();
        let mut messages = Vec::new();

        for i in 0..msg_count {
            let key = ((user, chat_id), i);
            if let Some((content, timestamp)) = map.get(&key) {
                messages.push((content.clone(), timestamp.clone()));
            }
        }

        messages
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
fn create_new_chat(user: Principal, chat_id: ChatId, name: String) {
    USER_CHATS.with(|user_chats| {
        let mut user_chats = user_chats.borrow_mut();
        let chats = user_chats.entry(user).or_insert_with(HashMap::new);
        chats.entry(chat_id).or_insert(ChatInfo {
            name,
            messages: Vec::new(),
        });
    });
}

#[update]
fn add_chat_message(user: Principal, chat_id: ChatId, question: String, answer: String) {
    USER_CHATS.with(|user_chats| {
        let mut user_chats = user_chats.borrow_mut();
        if let Some(chats) = user_chats.get_mut(&user) {
            if let Some(chat_info) = chats.get_mut(&chat_id) {
                chat_info.messages.push(ChatMessage { question, answer });
            }
        }
    });
}

#[query]
fn get_chat_history(user: Principal, chat_id: ChatId) -> ChatInfo {
    USER_CHATS.with(|user_chats| {
        user_chats.borrow()
            .get(&user)
            .and_then(|chats| chats.get(&chat_id))
            .cloned()
    }).expect("REASON")
}

#[update]
fn delete_chat(user: Principal, chat_id: ChatId) -> bool {
    USER_CHATS.with(|user_chats| {
        let mut user_chats = user_chats.borrow_mut();
        if let Some(chats) = user_chats.get_mut(&user) {
            return chats.remove(&chat_id).is_some();
        }
        false
    })
}

#[update]
fn rename_chat(user: Principal, chat_id: ChatId, new_name: String) -> bool {
    USER_CHATS.with(|user_chats| {
        let mut user_chats = user_chats.borrow_mut();
        if let Some(chats) = user_chats.get_mut(&user) {
            if let Some(chat_info) = chats.get_mut(&chat_id) {
                chat_info.name = new_name;
                return true;
            }
        }
        false
    })
}

#[query]
fn list_chats(user: Principal) -> Vec<ChatMeta> {
    USER_CHATS.with(|user_chats| {
        user_chats.borrow()
            .get(&user)
            .map(|chats| {
                chats.iter()
                    .map(|(chat_id, info)| ChatMeta {
                        id: chat_id.clone(),
                        name: info.name.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    })
}

#[update]
fn set_user_name(user: Principal, name: String) {
    USER_NAMES.with(|names| {
        names.borrow_mut().insert(user, name);
    });
}

#[query]
fn get_user_name(user: Principal) -> String {
    USER_NAMES.with(|names| {
        names
            .borrow()
            .get(&user)
            .cloned()
            .unwrap_or_else(|| "user".to_string())
    })
}

#[update]
pub fn try_increment_user_prompt(user: Principal) -> bool {
    let now = time();

    USER_PROMPTS.with(|map| {
        let mut map = map.borrow_mut();
        let entry = map.entry(user).or_insert((0, None));

        let (count, blocked_since) = entry;

        if let Some(block_time) = blocked_since {
            if now - *block_time >= BLOCK_TIME_NANOS {
                *count = 1;
                *blocked_since = None;
                return true;
            } else {
                return false;
            }
        } else {
            *count += 1;

            if *count >= PROMPT_LIMIT {
                *blocked_since = Some(now);
            }

            return true;
        }
    })
}