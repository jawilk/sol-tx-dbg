#[macro_use]
extern crate rocket;

use lazy_static::lazy_static;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

use rocket::request::FromParam;
use rocket::serde::json::{json, Value};
use rocket::serde::Serialize;

use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use solana_transaction_status::EncodedTransaction;
use solana_transaction_status::UiTransactionEncoding;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;

const SUPPORTED_PROGRAMS: [&'static str; 1] = ["ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"];
lazy_static! {
    static ref SUPPORTED_PROGRAMS_INFO: Mutex<HashMap<&'static str, ProgramMeta>> = {
        let mut m = HashMap::new();
        m.insert(
            SUPPORTED_PROGRAMS[0],
            ProgramMeta {
                name: "Associated Token Program",
            },
        );
        Mutex::new(m)
    };
}

struct TxHash {
    hash: String,
}

#[derive(Debug)]
enum TxHashError {
    Invalid,
}

impl<'r> FromParam<'r> for TxHash {
    type Error = TxHashError;
    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        match Signature::from_str(&param) {
            Ok(_) => Ok(TxHash {
                hash: param.to_string(),
            }),
            _ => Err(TxHashError::Invalid),
        }
    }
}

#[derive(Serialize, Copy, Clone)]
#[serde(crate = "rocket::serde")]
struct ProgramMeta {
    name: &'static str,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct InitResponse {
    is_supported: bool,
    program_info: Option<ProgramMeta>,
    program_id: String,
    /// To identify the request (i.e. account data etc.)
    tx_id: String,
}

fn get_tx_info(tx_hash_str: &str) -> Vec<InitResponse> {
    let mut programs = vec![];
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Json),
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };
    let tx_hash = Signature::from_str(&tx_hash_str).unwrap();
    let tx = rpc_client
        .get_transaction_with_config(&tx_hash, config)
        .unwrap()
        .transaction
        .transaction;
    if let EncodedTransaction::Json(tx) = tx {
        if let solana_transaction_status::UiMessage::Raw(message) = tx.message {
            for inst in message.instructions {
                let program_id = message.account_keys[inst.program_id_index as usize].clone();
                let is_supported = SUPPORTED_PROGRAMS.iter().any(|&p| p == program_id);
                programs.push(InitResponse {
                    is_supported,
                    program_info: if is_supported {
                        Some(
                            *SUPPORTED_PROGRAMS_INFO
                                .lock()
                                .unwrap()
                                .get(&*program_id)
                                .unwrap(),
                        )
                    } else {
                        None
                    },
                    program_id,
                    tx_id: tx_hash_str.to_string(),
                });
            }
        }
    }
    programs
}

#[get("/init/<tx_hash>")]
fn init(tx_hash: TxHash) -> Value {
    println!("hash here: {} {}", tx_hash.hash, tx_hash.hash.len());
    json!(get_tx_info(&tx_hash.hash))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![init]).attach(Cors)
}

pub struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Cross-Origin-Resource-Sharing Fairing",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, PATCH, PUT, DELETE, HEAD, OPTIONS, GET",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}
