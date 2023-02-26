#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;
use std::sync::Mutex;

use lazy_static::lazy_static;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

use rocket::http::RawStr;
use rocket::request::FromParam;

use rocket::serde::json::{json, Value};
use rocket::serde::Serialize;

use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use solana_transaction_status::EncodedTransaction;
use solana_transaction_status::UiTransactionEncoding;

use uuid::Uuid;

const SUPPORTED_PROGRAMS: [&'static str; 1] = ["ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"];
lazy_static! {
    static ref SUPPORTED_PROGRAMS_INFO: Mutex<HashMap<&'static str, String>> = {
        let mut m = HashMap::new();
        m.insert(
            SUPPORTED_PROGRAMS[0],
            String::from("Associated Token Program"),
        );
        Mutex::new(m)
    };
}

struct TxHash(String);

impl<'a> FromParam<'a> for TxHash {
    type Error = &'static str;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        match Signature::from_str(&param) {
            Ok(_) => Ok(Self {
                0: param.to_string(),
            }),
            Err(_) => Err("invalid TxHash"),
        }
    }
}

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
struct ProgramMeta {
    name: Option<String>,
    program_id: String,
    is_supported: bool,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct InitResponse {
    /// To identify the request (i.e. account data etc.)
    uuid: String,
    program_metas: Vec<ProgramMeta>,
}

fn get_tx_info(tx_hash_str: &str) -> InitResponse {
    let mut program_metas = vec![];
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
        // Save tx to disk to access later from POC
        let data = serde_json::to_vec(&tx).unwrap();
        match File::create(format!("../transactions/{tx_hash_str}.json")) {
            Ok(mut file) => file.write_all(&data).unwrap(),
            Err(e) => match e.kind() {
                std::io::ErrorKind::AlreadyExists => (),
                _ => panic!("Error create file {:?}", e),
            },
        };

        if let solana_transaction_status::UiMessage::Raw(message) = tx.message {
            for inst in message.instructions {
                let program_id = message.account_keys[inst.program_id_index as usize].clone();
                let is_supported = SUPPORTED_PROGRAMS.iter().any(|&p| p == program_id);
                program_metas.push(ProgramMeta {
                    name: if is_supported {
                        Some(
                            SUPPORTED_PROGRAMS_INFO
                                .lock()
                                .unwrap()
                                .get(&*program_id)
                                .unwrap()
                                .clone(),
                        )
                    } else {
                        None
                    },
                    program_id,
                    is_supported,
                });
            }
        }
    }
    InitResponse {
        uuid: Uuid::new_v4().to_string(),
        program_metas,
    }
}

#[get("/init/<tx_hash>")]
fn init(tx_hash: TxHash) -> Value {
    println!("hash here: {} {}", tx_hash.0, tx_hash.0.len());
    json!(get_tx_info(&tx_hash.0))
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
