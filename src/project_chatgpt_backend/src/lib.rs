use candid::{Principal, CandidType};
use core::arch;
use std::collections::HashMap;
use ic_cdk_macros::{update, query};
use serde::{Deserialize, Serialize};
use ic_cdk::api::time;
use ic_llm::{AssistantMessage, ChatBuilder, ChatMessage, Model};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap};
use std::cell::RefCell;
//use ic_stable_structures::storable::Storable;
//use ic_stable_structures::storable::Bound;

type Memory = VirtualMemory<DefaultMemoryImpl>;

const PROMPT_LIMIT: u32 = 250;
const BLOCK_TIME_NANOS: u64 = 12 * 60 * 60 * 1_000_000_000;

#[derive(Clone, CandidType, Deserialize, Serialize)]
struct ChatMeta {
    name: String,
    id: [u8; 16],
    msg_len: u32,
}

#[derive(Clone, CandidType, Deserialize)]
struct ChatMessageIC {
    role: String,
    content: String,
    etc: (u64, u32, u32),
}

#[derive(Clone, CandidType, Deserialize)]
struct ChatInfo {
    messages: Vec<ChatMessageIC>,
}

type ChatId = [u8; 16];

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static USER_NAMES_STABLE: RefCell<StableBTreeMap<Principal, [u8; 32], Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    static USER_PROMPTS_STABLE: RefCell<StableBTreeMap<Principal, (u32, [u8; 9]), Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
        )
    );

    static USER_CHATS_STABLE: RefCell<StableBTreeMap<(Principal, [u8; 16]), ([u8; 64], u32), Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    static USER_ARCHIVE_STABLE: RefCell<StableBTreeMap<(Principal, [u8; 16]), ([u8; 64], u32), Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
        )
    );

    static CHAT_MESSAGES_STABLE: RefCell<StableBTreeMap<((Principal, [u8; 16]), u32), ([u8; 32], [u8; 4096], (u64, u32, u32)), Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );
}

fn set_chat_archived_stable(user: Principal, chat_id: [u8; 16], archived: bool) -> bool {
    if archived {
        // przenieś do archiwum
        USER_CHATS_STABLE.with(|map| {
            let mut map = map.borrow_mut();
            if let Some((name, msg_count)) = map.remove(&(user, chat_id)) {
                USER_ARCHIVE_STABLE.with(|archive| {
                    archive
                        .borrow_mut()
                        .insert((user, chat_id), (name, msg_count));
                });
                true
            } else {
                false
            }
        })
    } else {
        // przenieś z archiwum z powrotem do normalnej mapy
        USER_ARCHIVE_STABLE.with(|archive| {
            let mut archive = archive.borrow_mut();
            if let Some((name, msg_count)) = archive.remove(&(user, chat_id)) {
                USER_CHATS_STABLE.with(|map| {
                    map.borrow_mut().insert((user, chat_id), (name, msg_count));
                });
                true
            } else {
                false
            }
        })
    }
}

fn option_f64_to_bytes(opt: Option<u64>) -> [u8; 9] {
    let mut bytes = [0u8; 9];
    match opt {
        None => bytes[0] = 0,
        Some(val) => {
            bytes[0] = 1;
            bytes[1..9].copy_from_slice(&val.to_le_bytes());
        }
    }
    bytes
}

fn bytes_to_option_f64(bytes: &[u8; 9]) -> Option<u64> {
    match bytes[0] {
        0 => None,
        1 => Some(u64::from_le_bytes(bytes[1..9].try_into().unwrap())),
        _ => None, // albo panic!("Invalid Option encoding")
    }
}

fn string_to_fixed_bytes<const N: usize>(s: &str) -> [u8; N] {
    let mut arr = [0u8; N];
    let bytes = s.as_bytes();
    let len = bytes.len().min(N);
    arr[..len].copy_from_slice(&bytes[..len]);
    arr
}

/// Zamienia tablicę bajtów na String, ignorując trailing zera.
fn fixed_bytes_to_string(bytes: &[u8]) -> String {
    let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8(bytes[..len].to_vec()).unwrap_or_default()
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
            blocked_since = bytes_to_option_f64(&b);
        }

        // logika blokady
        if let Some(block_time) = blocked_since {
            if now - block_time >= BLOCK_TIME_NANOS {
                // reset
                map.insert(user, (1, option_f64_to_bytes(None)));
                true
            } else {
                false
            }
        } else {
            count += 1;
            if count >= PROMPT_LIMIT {
                blocked_since = Some(now);
            }
            map.insert(user, (count, option_f64_to_bytes(blocked_since)));
            true
        }
    })
}

fn set_name_stable(principal: Principal, value: String) {
    USER_NAMES_STABLE.with(|map| map.borrow_mut().insert(principal, string_to_fixed_bytes::<32>(&value)));
}

fn get_name_stable(principal: Principal) -> Option<[u8; 32]> {
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
                    Some(ChatMeta {name: fixed_bytes_to_string(&name), id: index.clone(), msg_len: msg_count })
                } else {
                    None
                }
            })
            .collect()
    })
}

fn get_archives_for_user(user: Principal) -> Vec<ChatMeta> {
    USER_ARCHIVE_STABLE.with(|map_ref| {
        let map = map_ref.borrow();
        map.iter()
            .filter_map(|entry| {
                let (p, index) = entry.key();
                let (name, msg_count) = entry.value();
                if *p == user {
                    Some(ChatMeta {name: fixed_bytes_to_string(&name), id: index.clone(), msg_len: msg_count })
                } else {
                    None
                }
            })
            .collect()
    })
}

fn get_msgs_for_user(user: Principal, chat_id: [u8; 16], msg_count: u32) -> ChatInfo {
    CHAT_MESSAGES_STABLE.with(|map_ref| {
        let map = map_ref.borrow();
        let mut info = ChatInfo { messages: Vec::new() };

        for i in 0..msg_count {
            let key = ((user, chat_id.clone()), i);
            if let Some((role, content, etc)) = map.get(&key) {
                info.messages.push(ChatMessageIC { role: fixed_bytes_to_string(&role), content: fixed_bytes_to_string(&content), etc });
            }
        }

        info
    })
}

fn create_new_chat_stable(user: Principal, chat_id: [u8; 16], name: String) {
    USER_CHATS_STABLE.with(|map| {
        map.borrow_mut().insert((user, chat_id), (string_to_fixed_bytes::<64>(&name), 0));
    });
}

fn delete_chat_stable(user: Principal, chat_id: [u8; 16]) -> bool {
    let removed = USER_CHATS_STABLE.with(|map| map.borrow_mut().remove(&(user, chat_id.clone())));
    if removed.is_some() {
        CHAT_MESSAGES_STABLE.with(|map| {
            let mut map = map.borrow_mut();
            let keys_to_remove: Vec<_> = map
                .iter()
                .filter_map(|entry| {
                    let ((p, c), idx) = entry.key();
                    if *p == user && *c == chat_id {
                        Some(((user, chat_id.clone()), *idx))
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

fn rename_chat_stable(user: Principal, chat_id: [u8; 16], new_name: String) -> bool {
    USER_CHATS_STABLE.with(|map| {
        let mut map = map.borrow_mut();
        if let Some((_old_name, msg_count)) = map.get(&(user, chat_id.clone())) {
            map.insert((user, chat_id.clone()), (string_to_fixed_bytes::<64>(&new_name), msg_count));
            true
        } else {
            false
        }
    })
}

fn add_chat_message_stable(user: Principal, chat_id: [u8; 16], content: String, role: String, width: u32, height: u32, timestamp: u64) -> bool {
    let etc = (timestamp, width, height);
    USER_CHATS_STABLE.with(|chat_map| {
        let mut chat_map = chat_map.borrow_mut();
        if let Some((name, msg_count)) = chat_map.get(&(user, chat_id.clone())) {
            let new_index = msg_count;
            CHAT_MESSAGES_STABLE.with(|msg_map| {
                msg_map
                    .borrow_mut()
                    .insert(((user, chat_id.clone()), new_index), (string_to_fixed_bytes::<32>(&role), string_to_fixed_bytes::<4096>(content.as_str()), etc));
            });
            chat_map.insert((user, chat_id.clone()), (name.clone(), new_index + 1));
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
async fn chat(prompt: String, width: u32, height: u32, tag: String, user: Principal, chat_id: [u8; 16], msg_len: u32) -> String {
    let msgs = get_msgs_for_user(user, chat_id, msg_len);
    let rows = height;
    let cols = width;
    let new_prompt = prompt.clone();

    let mut max_history = 5;

    let model = match tag.as_str() {
        "Llama3_1_8B" => {
            Model::Llama3_1_8B
        }
        "Qwen3_32B" => {
            Model::Qwen3_32B
        }
        "Llama4Scout" => {
            Model::Llama4Scout
        }
        "Llama4Scout_Image" => {
            max_history = 4;
            Model::Llama4Scout
        }
        "Llama3_1_8B_Image" => {
            max_history = 4;
            Model::Llama3_1_8B
        }
        _ => {
            Model::Llama3_1_8B
        }
    };

    let mut messages = vec![];
    let history: Vec<_> = msgs.messages
        .iter()
        .rev()               // idziemy od końca
        .take(max_history)   // bierzemy tylko max_history
        .collect::<Vec<_>>() // tworzymy wektor
        .into_iter()
        .rev()               // przywracamy kolejność od starszych do nowszych
        .collect();
    for message in &history {
        match message.role.as_str() {
            "user" => {
                messages.push(
                    ChatMessage::User {
                        content: message.content.clone(),
                })
            }
            _ => {
                messages.push(
                    ChatMessage::Assistant (
                        AssistantMessage {
                            content: Some(message.content.clone()),
                            tool_calls: vec![], // jeśli nie używasz żadnych narzędzi
                        }
                ))
            }
        }
    }

    match tag.as_str() {
        "Llama4Scout_Image" | "Llama3_1_8B_Image" => {
            // budujemy specjalny system prompt dla HEX
            let sys_prompt = format!(
                "You are an AI that only outputs pixel-art in HEX grid format.

                Output rules (must be followed exactly):
                  1. Represent the image only as HEX color codes (#RRGGBB).
                  2. Each row must have exactly {cols} HEX codes.
                  3. Use exactly {rows} rows.
                  4. Do not include explanations or extra text.
                  5. After all rows, add: Summary: {rows} rows, each with {cols} HEXs (total {rows}x{cols} HEXs)."
            );

            let user_prompt = format!(
                "Generate, imaginate a pixel-art style {} image of size {rows}x{cols}.",
                prompt
            );

            messages.push(
                ChatMessage::System { content: sys_prompt },
            );

            messages.push(
                ChatMessage::User { content: user_prompt },
            );
        }
        _ => {
            messages.push(
                ChatMessage::User {
                    content: new_prompt.clone(),
            });
        }
    }

    let builder = ChatBuilder::new(model).with_messages(messages.clone());
    let response = builder.send().await;
    response.message.content.unwrap_or("ERR".to_string())
}

#[update]
fn create_new_chat(user: Principal, uid: [u8; 16], name: String) {
    create_new_chat_stable(user, uid, name);
}

#[update]
fn add_chat_message(user: Principal, chat_id: [u8; 16], content: String, role: String, width: u32, height: u32, date: u64) {
    let _ = add_chat_message_stable(user, chat_id, content, role, width, height, date);
}

#[query]
fn get_chat_history(user: Principal, chat_id: [u8; 16], msg_len: u32) -> ChatInfo {
    get_msgs_for_user(user, chat_id, msg_len)
}

#[update]
fn delete_chat(user: Principal, chat_id: [u8; 16]) -> bool {
    delete_chat_stable(user, chat_id)
}

#[update]
fn rename_chat(user: Principal, chat_id: [u8; 16], new_name: String) -> bool {
    rename_chat_stable(user, chat_id, new_name)
}

#[query]
fn list_chats(user: Principal, arch: bool) -> Vec<ChatMeta> {
    if arch {
        get_archives_for_user(user)
    } else {
        get_chats_for_user(user) 
    }
}

#[update]
fn set_user_name(user: Principal, name: String) {
    set_name_stable(user, name)
}

#[query]
fn get_user_name(user: Principal) -> String {
    get_name_stable(user).map(|b| fixed_bytes_to_string(&b))
    .unwrap_or_else(|| "anonimus".to_string())
}

#[update]
pub fn try_increment_user_prompt(user: Principal) -> bool {
    inc_user_prompt_stable(user)
}

#[update]
fn archive_chat(user: Principal, chat_id: [u8; 16], archive: bool) -> bool {
    set_chat_archived_stable(user, chat_id, archive)
}