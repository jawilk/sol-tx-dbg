#[macro_use]
extern crate rocket;

use anyhow::anyhow;
use lazy_static::lazy_static;
use log::debug;

use std::collections::HashMap;
use std::fs::File;
use std::fs::{self, DirEntry};
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;
use std::{thread, time};

use rocket::fairing::{Fairing, Info, Kind};
use rocket::fs::FileServer;
use rocket::http::Header;
use rocket::http::Status;
use rocket::request::FromParam;
use rocket::{Request, Response};

use rocket::serde::json::{json, Value};
use rocket::serde::{Deserialize, Serialize};
use serde_json::from_reader;

use uuid::Uuid;

use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::{ParseSignatureError, Signature};
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::EncodedTransaction;
use solana_transaction_status::UiTransactionEncoding;

lazy_static! {
    static ref SUPPORTED_PROGRAMS: HashMap<String, String> = {
        let file = File::open("static/supported_programs.json").unwrap();
        let reader = BufReader::new(file);
        let data: Vec<SupportedProgram> = from_reader(reader).unwrap();
        let mut map = HashMap::new();
        for d in data {
            map.insert(d.id, d.name);
        }
        map
    };
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct SupportedProgram {
    id: String,
    name: String,
}

struct TxHash(Signature);

impl<'a> FromParam<'a> for TxHash {
    type Error = ParseSignatureError;

    fn from_param(param: &'a str) -> Result<Self, Self::Error> {
        Ok(Self(Signature::from_str(param)?))
    }
}

#[derive(Serialize, Clone)]
#[serde(crate = "rocket::serde")]
struct ProgramMeta<'a> {
    name: Option<&'a String>,
    program_id: String,
    is_supported: bool,
    cpi_programs: Vec<String>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct InitResponse<'a> {
    /// To identify the request (e.g. account data etc.)
    uuid: String,
    tx_program_metas: Vec<ProgramMeta<'a>>,
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Program {
    id: String,
    is_supported: bool,
}

fn get_tx_programs(tx_hash: TxHash) -> anyhow::Result<Vec<Vec<String>>> {
    let tx_hash_str = tx_hash.0.to_string();
    let dir = format!("../cache/instructions/{tx_hash_str}");
    let path = Path::new(&dir);
    match fs::read_dir(path) {
        // Tx already exists
        Ok(files) => {
            debug!("Tx already cached!");
            let mut inst_files: Vec<DirEntry> = files.filter_map(Result::ok).collect();
            inst_files.sort_by_key(|dir_entry| dir_entry.file_name());
            Ok(inst_files
                .iter()
                .map(|inst| {
                    fs::read_to_string(inst.path())
                        .unwrap()
                        .lines()
                        .map(String::from)
                        .collect()
                })
                .collect())
        }
        // New tx, fetch from rpc
        Err(_) => {
            debug!("New tx!");
            let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
            let config = RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Json),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            };
            let now = Instant::now();
            let tx = loop {
                match rpc_client.get_transaction_with_config(&tx_hash.0, config) {
                    Ok(tx) => break tx.transaction,
                    _ => thread::sleep(time::Duration::from_secs(1)),
                }
                if now.elapsed().as_secs() > 5 {
                    return Err(anyhow!("Couldn't fetch transaction"));
                }
            };
            // Save tx to disk
            match tx.transaction {
                EncodedTransaction::Json(tx) => {
                    let data = serde_json::to_vec(&tx)?;
                    let mut file =
                        File::create(format!("../cache/transactions/{tx_hash_str}.json"))?;
                    // Save whole tx
                    file.write_all(&data)?;
                }
                _ => return Err(anyhow!("EncodedTransaction parse failed")),
            }
            let meta = tx
                .meta
                .ok_or_else(|| anyhow!("Cannot get transaction meta"))?;
            // Save all cpi programs per instruction to disk
            match meta.log_messages {
                OptionSerializer::Some(logs) => {
                    let mut tx_cpi_programs: Vec<Vec<String>> = vec![];
                    let mut current_group: Vec<String> = vec![];
                    for line in logs {
                        if line.contains("invoke [1]") {
                            tx_cpi_programs.push(current_group);
                            current_group = vec![];
                        }
                        if line.contains("invoke [") {
                            current_group.push(
                                line.split(" invoke ")
                                    .next()
                                    .unwrap()
                                    .split("Program ")
                                    .last()
                                    .unwrap()
                                    .to_string(),
                            );
                        }
                    }
                    tx_cpi_programs.push(current_group);
                    tx_cpi_programs.remove(0);
                    let folder = format!("../cache/instructions/{tx_hash_str}");
                    fs::create_dir(folder)?;
                    tx_cpi_programs
                        .iter()
                        .enumerate()
                        .for_each(|(inst_nr, vec)| {
                            let path = format!("../cache/instructions/{tx_hash_str}/{inst_nr}.txt");
                            std::fs::write(path, vec.join("\n")).unwrap()
                        });
                    Ok(tx_cpi_programs)
                }
                _ => return Err(anyhow!("Parsing message failed.")),
            }
        }
    }
}

fn get_tx_info<'a>(tx_hash: TxHash) -> anyhow::Result<InitResponse<'a>> {
    let tx_programs = get_tx_programs(tx_hash)?;
    let mut tx_program_metas = vec![];
    for mut inst_programs in tx_programs {
        let program_id = inst_programs.remove(0);
        tx_program_metas.push(ProgramMeta {
            name: SUPPORTED_PROGRAMS.get(&program_id),
            program_id: program_id.clone(),
            is_supported: SUPPORTED_PROGRAMS.contains_key(&program_id),
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
    match get_tx_info(tx_hash) {
        Ok(tx) => Ok(json!(tx)),
        Err(_) => Err(Status::InternalServerError),
    }
}

#[launch]
fn rocket() -> _ {
    env_logger::init();
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
