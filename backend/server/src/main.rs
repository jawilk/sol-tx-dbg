#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::fs::{self, DirEntry};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::str::FromStr;
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
use rocket::serde::{Deserialize, Serialize};
use serde_json::from_reader;

use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use solana_transaction_status::option_serializer::OptionSerializer;
use solana_transaction_status::EncodedTransaction;
use solana_transaction_status::UiTransactionEncoding;

use anyhow::anyhow;

use uuid::Uuid;

lazy_static! {
    static ref SUPPORTED_PROGRAMS: HashMap<String, String> = {
        let data = load_supported_programs("static/supported_programs.json");
        println!("supp: {:?}", data);
        let mut map = HashMap::new();
        for d in data {
            map.insert(d.id, d.name);
        }
        map
    };
}

fn load_supported_programs(file_path: &str) -> Vec<SupportedProgram> {
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let data = from_reader(reader).unwrap();
    data
}

#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct SupportedProgram {
    id: String,
    name: String,
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
                    let data = serde_json::to_vec(&tx).unwrap();
                    let mut file =
                        File::create(format!("../cache/transactions/{tx_hash_str}.json")).unwrap();
                    // Save whole tx
                    file.write_all(&data).unwrap();
                }
                _ => return Err(anyhow!("EncodedTransaction parse failed")),
            }
            let meta = tx.meta.unwrap();
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
                    let mut path = format!("../cache/instructions/{tx_hash_str}/0.txt");
                    let parent_dir = std::path::Path::new(&path).parent().unwrap();
                    create_dir_all(parent_dir)?;
                    for (inst_nr, vec) in tx_cpi_programs.iter().enumerate() {
                        path = format!("../cache/instructions/{tx_hash_str}/{inst_nr}.txt");
                        let mut file = File::create(path)?;
                        let content = vec.join("\n");
                        file.write_all(content.as_bytes()).unwrap();
                    }
                    Ok(tx_cpi_programs)
                }
                _ => return Err(anyhow!("Parsing message failed.")),
            }
        }
    }
}

fn get_tx_info(tx_hash_str: &str) -> anyhow::Result<InitResponse> {
    let tx_cpi_programs = load_tx(tx_hash_str).unwrap();
    let mut tx_program_metas = vec![];
    for mut inst_programs in tx_cpi_programs {
        let program_id = inst_programs.remove(0);
        let is_supported = SUPPORTED_PROGRAMS.contains_key(&program_id);
        tx_program_metas.push(ProgramMeta {
            name: if is_supported {
                Some(SUPPORTED_PROGRAMS.get(&program_id).unwrap().clone())
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
