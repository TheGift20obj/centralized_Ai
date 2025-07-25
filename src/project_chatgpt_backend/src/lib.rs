use ic_cdk::api::management_canister::http_request;
use ic_cdk::api::management_canister::http_request::HttpMethod;
use ic_cdk::api::management_canister::http_request::HttpResponse;
use ic_cdk::api::management_canister::http_request::CanisterHttpRequestArgument;
use ic_cdk::api::management_canister::http_request::HttpHeader;
use candid::{Principal, CandidType, Nat};
use std::collections::HashMap;
use ic_cdk_macros::{update, query};
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Clone, CandidType, Deserialize)]
struct ChatMessage {
    question: String,
    answer: String,
}

thread_local! {
    static USER_CHATS: std::cell::RefCell<HashMap<Principal, Vec<ChatMessage>>> = std::cell::RefCell::new(HashMap::new());
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

const CYCLES_FOR_HTTP_REQUEST: u128 = 12_000_000_000;

pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

#[update]
async fn chat(prompt: String) -> String {
    let openai_api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| "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".to_string());
    //second key: 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
    let body = json!({
        "model": "gpt-4o-mini",
        "messages": [{
            "role": "user",
            "context": prompt,
        }]
    });
    
    let body_str = body.to_string();

    let request = CanisterHttpRequestArgument {
        url: "https://api.openai.com/v1/chat/completions".to_string(),
        method: HttpMethod::POST,
        headers: vec![
            HttpHeader {
                name: "Authorization".to_string(),
                value: format!("Bearer {}", openai_api_key),
            },
            HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            },
        ],
        body: Some(body_str.into_bytes()),
        max_response_bytes: Some(1048576), // 1MB limit na odpowiedź
        transform: None,
    };

    match http_request::http_request(request, CYCLES_FOR_HTTP_REQUEST).await {
        Ok((HttpResponse { status, body, .. },)) if status == Nat::from(200u16) => {
            //return "Test".to_string();
            let resp_str = String::from_utf8(body).unwrap_or_default();
            // Parsujemy JSON OpenAI response aby zwrócić treść odpowiedzi:
            #[derive(Deserialize)]
            struct Choice {
                message: Message,
            }
            #[derive(Deserialize)]
            struct OpenAIResponse {
                choices: Vec<Choice>,
            }

            match serde_json::from_str::<OpenAIResponse>(&resp_str) {
                Ok(resp) => {
                    if let Some(choice) = resp.choices.first() {
                        choice.message.content.clone()
                    } else {
                        "No choices in response".to_string()
                    }
                }
                Err(e) => format!("JSON parse error: {}", e),
            }
        }
        Ok((HttpResponse { status, body, .. },)) => {
            let error_body = String::from_utf8_lossy(&body);
            format!("OpenAI error status: {}, body: {}", status, error_body)
        }
        Err(e) => format!("HTTP request error: {:?}", e),
    }
}

#[update]
fn add_chat_message(user: Principal, question: String, answer: String) {
    USER_CHATS.with(|chats| {
        let mut chats = chats.borrow_mut();
        let entry = chats.entry(user).or_insert_with(Vec::new);
        entry.push(ChatMessage { question, answer });
    });
}

#[query]
fn get_chat_history(user: Principal) -> Vec<ChatMessage> {
    USER_CHATS.with(|chats| {
        chats.borrow().get(&user).cloned().unwrap_or_default()
    })
}