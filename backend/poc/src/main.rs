use anyhow::anyhow;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::{BufRead, BufReader};
use std::{env, str::FromStr};

use poc_framework::{Environment, LocalEnvironment};
use solana_bpf_loader_program::set_port;
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::{AccountMeta, Instruction};
use solana_program::{native_token::sol_to_lamports, pubkey::Pubkey, system_program};
use solana_transaction_status::UiRawMessage;
use solana_transaction_status::UiTransaction;

const SUPPORTED_PROGRAMS: [&'static str; 1] =
    ["ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL, TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"];

pub fn main() {
    let args: Vec<_> = env::args().collect();
    let tx_hash = &args[1];
    let inst_nr = args[2].parse::<usize>().unwrap();
    let port = args[3].parse::<u16>().unwrap();
    println!("id: {} nr: {} port: {}", tx_hash, inst_nr, port);
    setup(tx_hash, inst_nr, port);
}

fn create_single_inst(
    message: &UiRawMessage,
    inst_nr: usize,
    writable_accs: Vec<u8>,
    signer_accs: Vec<u8>,
) -> Instruction {
    let inst = &message.instructions[inst_nr];
    let inst_index = inst.program_id_index as usize;
    let mut accounts = vec![];
    for i in inst.accounts.iter() {
        accounts.push(AccountMeta {
            pubkey: Pubkey::from_str(&message.account_keys[*i as usize]).unwrap(),
            is_signer: signer_accs.contains(i),
            is_writable: writable_accs.contains(i),
        });
    }
    Instruction {
        program_id: Pubkey::from_str(&message.account_keys[inst_index]).unwrap(),
        accounts,
        data: inst.data.as_bytes().to_vec(),
    }
}

fn load_tx(tx_hash: &str) -> UiTransaction {
    if let Ok(mut file) = File::open(format!("../cache/transactions/{tx_hash}.json")) {
        let mut buf = vec![];
        if file.read_to_end(&mut buf).is_ok() {
            match serde_json::from_slice::<UiTransaction>(&buf[..]) {
                Ok(tx) => return tx,
                _ => panic!("Error deserializing tx"),
            }
        } else {
            panic!("Error reading file!");
        }
    } else {
        panic!("Error opening file!");
    }
}

fn get_inst(
    tx_hash_str: &str,
    inst_nr: usize,
) -> anyhow::Result<(Instruction, Vec<String>, Vec<String>, Pubkey)> {
    // Load tx from disk
    let tx = load_tx(tx_hash_str);
    match tx.message {
        solana_transaction_status::UiMessage::Raw(message) => {
            println!("msg: {:?}", message);
            let payer = Pubkey::from_str(&message.account_keys[0]).unwrap();
            let mut writable_accs = (0..message.header.num_required_signatures
                - message.header.num_readonly_signed_accounts)
                .collect::<Vec<_>>();
            writable_accs.extend(
                (message.header.num_required_signatures
                    + message.header.num_readonly_signed_accounts
                    ..message.account_keys.len() as u8
                        - message.header.num_readonly_unsigned_accounts)
                    .collect::<Vec<_>>(),
            );
            let signer_accs = (0..message.header.num_required_signatures
                + message.header.num_readonly_signed_accounts)
                .collect::<Vec<_>>();

            let inst_file = format!("../cache/instructions/{tx_hash_str}/{inst_nr}.txt");
            let file = fs::File::open(inst_file)?;
            let reader = BufReader::new(file);
            let mut programs_supported = vec![];
            let mut programs_not_supported = vec![];
            for program in reader.lines() {
                let program = program?;
                if program != system_program::id().to_string() {
                    let is_supported = SUPPORTED_PROGRAMS.iter().any(|&p| p == program);
                    if is_supported {
                        programs_supported.push(program);
                    } else {
                        programs_not_supported.push(program);
                    }
                }
            }

            Ok((
                create_single_inst(&message, inst_nr, writable_accs, signer_accs),
                programs_supported,
                programs_not_supported,
                payer,
            ))
        }
        _ => return Err(anyhow!("Parsing message failed.")),
    }
}

fn sanitize_accounts(
    accounts: &Vec<AccountMeta>,
    programs_supported: &Vec<String>,
    programs_not_supported: &Vec<String>,
    accounts_avoid: &Vec<String>,
) -> Vec<Pubkey> {
    let mut accounts_sanitized = vec![];
    for acc in accounts.iter() {
        if !programs_supported.contains(&acc.pubkey.to_string())
            && !programs_not_supported.contains(&acc.pubkey.to_string())
            && !accounts_avoid.contains(&acc.pubkey.to_string())
        {
            accounts_sanitized.push(acc.pubkey);
        }
    }
    accounts_sanitized
}

fn setup(tx_hash: &str, inst_nr: usize, port: u16) {
    println!("PORT framework: {}", port);
    set_port(port);

    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());

    let (inst, programs_supported, programs_not_supported, payer) =
        get_inst(tx_hash, inst_nr).unwrap();

    let mut accounts_avoid = vec![];
    accounts_avoid.push("7NrmtYT7DsviTAzfpfgxPzywabGALCtz5mrxei3zCa5v".to_owned());

    let mut env = LocalEnvironment::builder()
        .clone_accounts_from_cluster(
            &sanitize_accounts(
                &inst.accounts,
                &programs_supported,
                &programs_not_supported,
                &accounts_avoid,
            ),
            &rpc_client,
        )
        .add_programs_supported(&programs_supported)
        .add_programs_not_supported(&programs_not_supported, &rpc_client)
        // // Add the original payer
        .add_account_with_lamports(payer, system_program::ID, sol_to_lamports(1.0))
        .build();

    println!("payer: {:?}", payer);
    println!("inst: {:?}", inst);

    env.execute_as_transaction_unsigned(&[inst], &payer);
}
