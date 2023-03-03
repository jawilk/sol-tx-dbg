#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Instant;
use std::{thread, time};

use lazy_static::lazy_static;

use rocket::fairing::{Fairing, Info, Kind};
use rocket::fs::FileServer;
use rocket::http::Header;
use rocket::http::Status;
use rocket::request::FromParam;
use rocket::{Request, Response};

use rocket::serde::json::{json, Value};
use rocket::serde::Serialize;

use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_transaction_status::EncodedTransaction;
use solana_transaction_status::UiTransaction;
use solana_transaction_status::UiTransactionEncoding;

use anyhow::anyhow;

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
    cpi_programs: Vec<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct InitResponse {
    /// To identify the request (e.g. account data etc.)
    uuid: String,
    program_metas: Vec<ProgramMeta>,
}

fn load_tx(tx_hash_str: &str, rpc_client: &RpcClient) -> anyhow::Result<UiTransaction> {
    let tx = match File::open(format!("../transactions/{tx_hash_str}.json")) {
        // Tx already exists
        Ok(mut file) => {
            println!("Tx already cached!");
            let mut buf = vec![];
            file.read_to_end(&mut buf).unwrap();
            serde_json::from_slice::<UiTransaction>(&buf[..])?
        }
        // New tx, fetch from rpc
        Err(_) => {
            println!("New tx!");
            let config = RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Json),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            };
            let tx_hash = Signature::from_str(&tx_hash_str).unwrap();
            let now = Instant::now();
            let tx = loop {
                match rpc_client.get_transaction_with_config(&tx_hash, config) {
                    Ok(tx) => break tx.transaction.transaction,
                    _ => thread::sleep(time::Duration::from_secs(1)),
                }
                if now.elapsed().as_secs() > 5 {
                    return Err(anyhow!("Couldn't fetch transaction"));
                }
            };
            match tx {
                EncodedTransaction::Json(tx) => {
                    let data = serde_json::to_vec(&tx).unwrap();
                    let mut file =
                        File::create(format!("../transactions/{tx_hash_str}.json")).unwrap();
                    file.write_all(&data).unwrap();
                    tx
                }
                _ => return Err(anyhow!("EncodedTransaction parse failed")),
            }
        }
    };
    Ok(tx)
}

fn get_tx_info(tx_hash_str: &str) -> anyhow::Result<InitResponse> {
    let mut program_metas = vec![];
    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());

    let tx = load_tx(tx_hash_str, &rpc_client).unwrap();

    match tx.message {
        solana_transaction_status::UiMessage::Raw(message) => {
            println!("msg: {:?}", message);
            for inst in message.instructions {
                let mut cpi_programs = vec![];
                for account_index in inst.accounts.iter() {
                    let account = &message.account_keys[*account_index as usize];
                    let account_info = rpc_client.get_account(&Pubkey::from_str(account).unwrap());
                    match account_info {
                        Ok(account_info) => {
                            if account_info.executable {
                                if !cpi_programs.contains(account) {
                                    cpi_programs.push(account.clone());
                                }
                            }
                        }
                        _ => (),
                    }
                }
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
                    cpi_programs,
                });
            }
        }
        _ => return Err(anyhow!("Parsing message failed.")),
    }
    Ok(InitResponse {
        uuid: Uuid::new_v4().to_string(),
        program_metas,
    })
}

#[get("/init/<tx_hash>")]
fn init(tx_hash: TxHash) -> Result<Value, Status> {
    println!("hash here: {} {}", tx_hash.0, tx_hash.0.len());
    match get_tx_info(&tx_hash.0) {
        Ok(tx) => Ok(json!(tx)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![init])
        .mount("/static", FileServer::from("static/"))
        .attach(Cors)
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
