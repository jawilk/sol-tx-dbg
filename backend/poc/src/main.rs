use std::{env, str::FromStr};

use poc_framework::{Environment, LocalEnvironment};
use solana_client::rpc_client::RpcClient;
use solana_transaction_status::UiTransaction;

use solana_transaction_status::UiRawMessage;

// use solana_program::native_token::lamports_to_sol;
use solana_program::{native_token::sol_to_lamports, pubkey::Pubkey, system_program};

use solana_program::instruction::{AccountMeta, Instruction};
// use spl_token_2022::extension::StateWithExtensions;
// use spl_token_2022::state::Account;

use solana_bpf_loader_program::set_port;
use std::fs::File;

use std::io::Read;

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

// fn get_signers(accounts: &Vec<AccountMeta>) -> Vec<&Pubkey> {
//     let mut signers = vec![];
//     for acc in accounts.iter() {
//         if acc.is_signer {
//             signers.push(&acc.pubkey);
//         }
//     }
//     signers
// }

fn sanitize_accounts(accounts: &Vec<AccountMeta>, programs: &Vec<Pubkey>) -> Vec<Pubkey> {
    let mut accounts_sanitized = vec![];
    for acc in accounts.iter() {
        if !programs.contains(&acc.pubkey) {
            accounts_sanitized.push(acc.pubkey);
        }
    }
    accounts_sanitized
}

fn load_tx(tx_hash: &str) -> UiTransaction {
    // Read
    if let Ok(mut file) = File::open(format!("../transactions/{tx_hash}.json")) {
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

fn get_inst(tx_hash: &str, inst_nr: usize) -> (Instruction, Pubkey) {
    // load whole instruction from disc
    let tx = load_tx(tx_hash);
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
            (
                create_single_inst(&message, inst_nr, writable_accs, signer_accs),
                payer,
            )
        }
        _ => panic!("NO INST FOUND"),
    }
}

fn setup(tx_hash: &str, inst_nr: usize, port: u16) {
    println!("PORT RUST: {}", port); // Set debugger port
    set_port(port); // - 1);

    let (inst, payer) = get_inst(tx_hash, inst_nr);

    // let helloworld_program =
    // Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap();
    // let mut dir = env::current_exe().unwrap();
    // let path_hello_world_binary = {
    // dir.pop();
    // dir.pop();
    // dir.pop();
    // dir.push("tests/elfs/associated.so");
    // dir.to_str()
    // }
    // .unwrap();

    let token_pubkey = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();

    let rpc_client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
    println!(
        "loadding-elf: {}",
        format!("tests/elfs/{}.so", inst.program_id)
    );

    // TODO: get all in inst involved program IDs
    let mut programs = vec![];
    programs.push(inst.program_id);
    programs.push(token_pubkey);

    let mut env = LocalEnvironment::builder()
        .clone_accounts_from_cluster(&sanitize_accounts(&inst.accounts, &programs), &rpc_client)
        .add_program(
            inst.program_id,
            format!("tests/elfs/{}.so", inst.program_id),
        )
        .add_program(
            token_pubkey,
            format!(
                "tests/elfs/{}.so",
                "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
            ),
        )
        // // Add the original payer
        .add_account_with_lamports(payer, system_program::ID, sol_to_lamports(1.0))
        .build();

    println!("payer: {:?}", payer);
    //    if let Ok(associated_token_account) = StateWithExtensions::<Account>::unpack(
    //      &env.get_account(inst.accounts[1].pubkey).unwrap().data,
    //) {
    //  println!("Account owner: {:?}", associated_token_account);
    println!("inst: {:?}", inst);
    env.execute_as_transaction_unsigned(&[inst], &payer);
    //}
    /*
    let mut dir = env::current_exe().unwrap();
    let path_hello_world_binary = {
        dir.pop();
        dir.pop();
        //dir.push("deploy");
        //dir.push("helloworld_rust_unoptimized.so");
        dir.pop();
        dir.push("tests/elfs/hello.so");
        dir.to_str()
    }
    .unwrap();
    let a_lot_of_money = sol_to_lamports(1_000_000.0);

    let helloworld_program =
        Pubkey::from_str("H311ot3333333333333333333333333333333333333").unwrap();
    let payer = keypair(0);
    let greeting_account = keypair(1);
    let data: [u8; 4] = [0; 4];

    //let a_lot_of_money = sol_to_lamports(1_000_000.0);

    let mut env = LocalEnvironment::builder()
        .add_program(helloworld_program, path_hello_world_binary)
        // .add_programs_to_debug(&[&helloworld_program])
        .add_account_with_lamports(payer.pubkey(), system_program::ID, sol_to_lamports(1.0))
        .add_account_with_data(greeting_account.pubkey(), helloworld_program, &data, false)
        .build();

    // env.execute_as_transaction_unsigned(
    // &[Instruction {
    // program_id: helloworld_program,
    // accounts: vec![AccountMeta::new(greeting_account.pubkey(), true)],
    // data: vec![0, 0, 0],
    // }],
    // &payer.pubkey(),
    // )
    env.execute_as_transaction(
        &[Instruction {
            program_id: helloworld_program,
            accounts: vec![AccountMeta::new(greeting_account.pubkey(), true)],
            data: vec![0, 0, 0, 0],
        }],
        &[&greeting_account],
    )
    .print();*/
}
