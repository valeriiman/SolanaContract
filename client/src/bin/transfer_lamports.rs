use std::{env::var, str::FromStr};

use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey, 
    transaction::Transaction,
    signature::{Keypair, Signer},

};
use solana_program::{
    system_program,
    instruction::{Instruction, AccountMeta},
};

use multisend::processor::{MultisendInstruction, Managment};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn establish_connection(rpc_url: String) -> Result<RpcClient> {
    Ok(RpcClient::new(rpc_url))
}

fn main() {
    let client = establish_connection(var("RPC_URL").expect("Missing RPC_URL")).unwrap();
    
    
    // let payer = Keypair::read_from_file(".config/solana/id.json").unwrap(); 
    let secret_key: [u8; 64] = [];
    let payer = Keypair::from_bytes(&secret_key).unwrap();


    let contract = Pubkey::from_str(&var("CONTRACT").unwrap()).unwrap();
    let bank = Pubkey::from_str("ET6JPT6EEVQXtofs8Rvrb4Fo9wXFfLxiTBQvXPncXg4k").unwrap();
    let (settings_pubkey, _) = Managment::get_settings_pubkey_with_bump(&contract);

    let recent_hash = client.get_latest_blockhash().unwrap();

    let instruction = Instruction::new_with_borsh(
        contract,
        &MultisendInstruction::SendLamports { amounts: vec![2000000] },
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(settings_pubkey, false),
            AccountMeta::new(bank, false),               
            AccountMeta::new(payer.pubkey(), false),
            AccountMeta::new(system_program::id(), false)
        ],
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        recent_hash
    );

    let x = client.send_and_confirm_transaction(&transaction);
    println!("{x:?}");

}