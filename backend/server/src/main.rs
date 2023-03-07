#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::fs::{self, DirEntry};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;
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
use solana_transaction_status::UiTransactionEncoding;

use anyhow::anyhow;

use uuid::Uuid;

const SUPPORTED_PROGRAMS: [&str; 2] = [
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
];
lazy_static! {
    static ref SUPPORTED_PROGRAMS_INFO: Mutex<HashMap<&'static str, String>> = {
        let mut m = HashMap::new();
        m.insert(
            SUPPORTED_PROGRAMS[0],
            String::from("Associated Token Program"),
        );
        m.insert(
            SUPPORTED_PROGRAMS[1],
            String::from("Token Program"),
        );
        Mutex::new(m)
    };
}

struct TxHash(String);

impl<'a> FromParam<'a> for TxHash {
    type Error = &'static str;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        match Signature::from_str(param) {
            Ok(_) => Ok(Self(param.to_string())),
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
    tx_program_metas: Vec<ProgramMeta>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Program {
    id: String,
    is_supported: bool,
}

fn load_tx(tx_hash_str: &str) -> anyhow::Result<Vec<Vec<String>>> {
    let dir = format!("../cache/instructions/{tx_hash_str}");
    let path = Path::new(&dir);
    match fs::read_dir(path) {
        // Tx already exists
        Ok(files) => {
            println!("Tx already cached!");
            let mut inst_files: Vec<DirEntry> = files.filter_map(Result::ok).collect();
            inst_files.sort_by_key(|dir_entry| dir_entry.file_name());
            let mut tx_programs = vec![];
            for inst in inst_files {
                println!("Inst nr: {:?}", inst);
                if inst.file_type()?.is_file() {
                    let file_path = inst.path();
                    let file = fs::File::open(file_path)?;
                    let reader = BufReader::new(file);
                    let mut inst_programs = vec![];
                    for program in reader.lines() {
                        inst_programs.push(program?);
                    }
                    tx_programs.push(inst_programs);
                }
            }
            Ok(tx_programs)
        }
        // New tx, fetch from rpc
        Err(_) => {
            println!("New tx!");
            let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
            let config = RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Json),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            };
            let tx_hash = Signature::from_str(tx_hash_str).unwrap();
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
                        File::create(format!("../cache/transactions/{tx_hash_str}.json")).unwrap();
                    // Save whole tx
                    file.write_all(&data).unwrap();
                    match tx.message {
                        solana_transaction_status::UiMessage::Raw(message) => {
                            println!("msg: {:?}", message);
                            // Save all programs used in an instruction separately
                            let mut tx_programs = vec![];
                            for (inst_nr, inst) in message.instructions.iter().enumerate() {
                                let mut inst_programs = vec![];
                                for account_index in inst.accounts.iter() {
                                    let account = &message.account_keys[*account_index as usize];
                                    let account_info =
                                        rpc_client.get_account(&Pubkey::from_str(account).unwrap());

                                    if let Ok(account_info) = account_info {
                                        if account_info.executable
                                            && !inst_programs.contains(&account.to_string())
                                        {
                                            inst_programs.push(account.to_string());
                                        }
                                    };
                                }
                                let program_id =
                                    message.account_keys[inst.program_id_index as usize].clone();
                                // Last program is always the main one
                                inst_programs.push(program_id.to_string());
                                let path =
                                    format!("../cache/instructions/{tx_hash_str}/{inst_nr}.txt");
                                let parent_dir = std::path::Path::new(&path).parent().unwrap();
                                create_dir_all(parent_dir)?;
                                let mut file = File::create(path)?;
                                for program in inst_programs.iter() {
                                    writeln!(file, "{}", program).unwrap();
                                }
                                tx_programs.push(inst_programs);
                            }
                            Ok(tx_programs)
                        }
                        _ => return Err(anyhow!("Parsing message failed.")),
                    }
                }
                _ => return Err(anyhow!("EncodedTransaction parse failed")),
            }
        }
    }
}

fn get_tx_info(tx_hash_str: &str) -> anyhow::Result<InitResponse> {
    let tx_programs = load_tx(tx_hash_str).unwrap();
    let mut tx_program_metas = vec![];
    for mut inst_programs in tx_programs {
        let program_id = inst_programs.pop().unwrap();
        let is_supported = SUPPORTED_PROGRAMS.iter().any(|&p| p == program_id);
        tx_program_metas.push(ProgramMeta {
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
            cpi_programs: inst_programs,
        });
    }
    Ok(InitResponse {
        uuid: Uuid::new_v4().to_string(),
        tx_program_metas,
    })
}

#[get("/tx-info/<tx_hash>")]
fn tx_info(tx_hash: TxHash) -> Result<Value, Status> {
    println!("hash here: {} {}", tx_hash.0, tx_hash.0.len());
    match get_tx_info(&tx_hash.0) {
        Ok(tx) => Ok(json!(tx)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", FileServer::from("dist").rank(1))
        .mount("/static", FileServer::from("static").rank(2))
        .mount("/", routes![tx_info])
        .mount("/choose", FileServer::from("dist").rank(2))
        .mount("/program", FileServer::from("dist").rank(2))
        .mount("/program/not-supported", FileServer::from("dist").rank(3))
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
