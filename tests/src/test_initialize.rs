use std::str::FromStr;

use anchor_client::{
    Client, Cluster, anchor_lang::solana_program::example_mocks::solana_sdk::system_program, solana_sdk::{
        commitment_config::CommitmentConfig, pubkey::Pubkey, signature::read_keypair_file, signer::Signer,
    }
};

#[test]
fn test_initialize() {
    let program_id = "BHqZnZvzQ9ogmBJKjSzKEukqwxXCrgWgqsaDNbU6QkGw";
    let anchor_wallet = std::env::var("ANCHOR_WALLET").unwrap();
    let payer = read_keypair_file(&anchor_wallet).unwrap();

    let client = Client::new_with_options(Cluster::Localnet, &payer, CommitmentConfig::confirmed());
    let program_id = Pubkey::from_str(program_id).unwrap();
    let program = client.program(program_id).unwrap();

    let tx = program
        .request()
        .accounts(solana_contracts::accounts::Initialize {
            counter: Pubkey::find_program_address(&[b"counter"], &program_id).0,
            payer: payer.pubkey(),
            system_program: system_program::id(),
        })
        .args(solana_contracts::instruction::Initialize {})
        .send()
        .expect("");

    println!("Your transaction signature {}", tx);

    let incriment_tx = program
        .request()
        .accounts(solana_contracts::accounts::Increment {
            counter: Pubkey::find_program_address(&[b"counter"], &program_id).0,
        })
        .args(solana_contracts::instruction::Increment {})
        .send()
        .expect("");

    println!("Your transaction signature {}", incriment_tx);
}
