use candid::{CandidType, Deserialize};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};

mod types;
mod nlp;
mod response;

use types::{ChatAnalysis};
use nlp::analyze_message;
use response::generate_response;

#[query]
async fn chat(message: String) -> String {
    let analysis: ChatAnalysis = analyze_message(&message);
    let reply = generate_response(&analysis);

    reply
}