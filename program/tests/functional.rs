use multisend::processor::Managment;

use {
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_program,
        rent::Rent,
        system_instruction,
        program_pack::Pack,
        sysvar,
        // msg
    },
    solana_program_test::*,
    solana_sdk::{account::Account as SolanaAccount, transaction::Transaction},
    solana_sdk::signature::{Signer, Keypair},

    multisend::processor::{ process_instruction, MultisendInstruction },
    std::str::FromStr,
    spl_token::state::Account

};


#[tokio::test]
async fn test_transfer_lamports_success() {

    
    let program_id = Pubkey::from_str("Transfer11111111111111111111111111111111111").unwrap();

    let sender = Keypair::new();
    let receiver_1 = Keypair::new();
    let receiver_2 = Keypair::new();
    let bank = Keypair::new();    

    let mut program_test = ProgramTest::new("multisend", program_id, processor!(process_instruction));

    program_test.add_account(
        sender.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );
    program_test.add_account(
        receiver_1.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );
    program_test.add_account(
        receiver_2.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );   

    program_test.add_account(
        bank.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );         

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let (settings_pubkey, _) = Managment::get_settings_pubkey_with_bump(&program_id);
    let tx = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &MultisendInstruction::UpdateSettings { fee: 10 } ,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(settings_pubkey, false),
                AccountMeta::new(bank.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash
    );
    banks_client.process_transaction(tx).await.unwrap();


    let tx = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &MultisendInstruction::SendLamports { amounts: vec![100000, 200000] },
            vec![
                AccountMeta::new(sender.pubkey(), true),
                AccountMeta::new(settings_pubkey, false),
                AccountMeta::new(bank.pubkey(), false),               
                AccountMeta::new(receiver_1.pubkey(), false),
                AccountMeta::new(receiver_2.pubkey(), false),
                AccountMeta::new(system_program::id(), false)
            ],
        )],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash
    );
    banks_client.process_transaction(tx).await.unwrap();

    let bank_account = banks_client
        .get_account(bank.pubkey())
        .await
        .expect("get_account")
        .expect("sender account not found");
    let bank_lamports = bank_account.lamports;

    let receiver_1_account = banks_client
        .get_account(receiver_1.pubkey())
        .await
        .expect("get_account")
        .expect("sender account not found");
    let receiver_1_lamports = receiver_1_account.lamports;

    let receiver_2_account = banks_client
        .get_account(receiver_2.pubkey())
        .await
        .expect("get_account")
        .expect("sender account not found");
    let receiver_2_lamports = receiver_2_account.lamports;

    assert_eq!(bank_lamports, 10_000_300);
    assert_eq!(receiver_1_lamports, 10_100_000);
    assert_eq!(receiver_2_lamports, 10_200_000);

}


#[tokio::test]
async fn test_transfer_tokens() {
    // Setup some pubkeys for the accounts
    let program_id = Pubkey::from_str("Transfer11111111111111111111111111111111111").unwrap();
    let source = Keypair::new();
    let mint = Keypair::new();
    let reciver1 = Keypair::new();
    let reciver2 = Keypair::new();
    let (authority_pubkey, _bump_seed) = Pubkey::find_program_address(&[b"authority"], &program_id);
    let bank = Keypair::new();    

    // Add the program to the test framework
    let program_test = ProgramTest::new(
        "multisend",
        program_id,
        processor!(process_instruction),
    );
    let amount = 1_000_000;
    let decimals = 9;
    let rent = Rent::default();

    // Start the program test
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    // Setup the mint, used in `spl_token::instruction::transfer_checked`
    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &mint.pubkey(),
                rent.minimum_balance(82),
                82 as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_mint(
                &spl_token::id(),
                &mint.pubkey(),
                &payer.pubkey(),
                None,
                decimals,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &mint],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    // Setup the source account, owned by the program-derived address
    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &source.pubkey(),
                rent.minimum_balance(165),
                165 as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &source.pubkey(),
                &mint.pubkey(),
                &authority_pubkey,
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &source],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();

    // Setup the destination account, used to receive tokens from the account
    // owned by the program-derived address
    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &reciver1.pubkey(),
                rent.minimum_balance(165),
                165 as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &reciver1.pubkey(),
                &mint.pubkey(),
                &payer.pubkey(),
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &reciver1],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();


    // Setup the destination account, used to receive tokens from the account
    // owned by the program-derived address
    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &reciver2.pubkey(),
                rent.minimum_balance(165),
                165 as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &reciver2.pubkey(),
                &mint.pubkey(),
                &payer.pubkey(),
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &reciver2],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();



    // Setup the destination account, used to receive tokens from the account
    // owned by the program-derived address
    let transaction = Transaction::new_signed_with_payer(
        &[
            system_instruction::create_account(
                &payer.pubkey(),
                &bank.pubkey(),
                rent.minimum_balance(165),
                165 as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &bank.pubkey(),
                &mint.pubkey(),
                &payer.pubkey(),
            )
            .unwrap(),
        ],
        Some(&payer.pubkey()),
        &[&payer, &bank],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();    


    // Mint some tokens to the PDA account
    let transaction = Transaction::new_signed_with_payer(
        &[spl_token::instruction::mint_to(
            &spl_token::id(),
            &mint.pubkey(),
            &source.pubkey(),
            &payer.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();



    // Check that the source account now has `amount` tokens
    let check_account = banks_client
        .get_account(source.pubkey())
        .await
        .unwrap()
        .unwrap();
    let token_account = Account::unpack(&check_account.data).unwrap();
    assert_eq!(token_account.amount, amount);



    let (settings_pubkey, _) = Managment::get_settings_pubkey_with_bump(&program_id);
    let tx = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &MultisendInstruction::UpdateSettings { fee: 10 } ,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(settings_pubkey, false),
                AccountMeta::new(bank.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash
    );
    banks_client.process_transaction(tx).await.unwrap();



    // Create an instruction following the account order expected by the program
    let transaction = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &MultisendInstruction::SendTokens { amounts: vec![100000, 200000] },
            vec![                
                AccountMeta::new(source.pubkey(), false),
                AccountMeta::new(settings_pubkey, false),
                AccountMeta::new(bank.pubkey(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(mint.pubkey(), false),
                AccountMeta::new_readonly(authority_pubkey, false),                
                AccountMeta::new(reciver1.pubkey(), false), 
                AccountMeta::new(reciver2.pubkey(), false),                
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    banks_client.process_transaction(transaction).await.unwrap();


    // Check that the destination account now has `amount` tokens
    let account = banks_client
        .get_account(bank.pubkey())
        .await
        .unwrap()
        .unwrap();
    let bank_account = Account::unpack(&account.data).unwrap();    

    // Check that the destination account now has `amount` tokens
    let account = banks_client
        .get_account(reciver1.pubkey())
        .await
        .unwrap()
        .unwrap();
    let reciver1_account = Account::unpack(&account.data).unwrap();

    // Check that the destination account now has `amount` tokens
    let account = banks_client
        .get_account(reciver2.pubkey())
        .await
        .unwrap()
        .unwrap();
    let reciver2_account = Account::unpack(&account.data).unwrap();

    assert_eq!(bank_account.amount, 300);
    assert_eq!(reciver1_account.amount, 100000);
    assert_eq!(reciver2_account.amount, 200000);


}



#[tokio::test]
async fn test_transfer_lamports_update_fee() {
    
    let program_id = Pubkey::from_str("Transfer11111111111111111111111111111111111").unwrap();

    let sender = Keypair::new();
    let receiver_1 = Keypair::new();
    let receiver_2 = Keypair::new();
    let bank = Keypair::new();    

    let mut program_test = ProgramTest::new("multisend", program_id, processor!(process_instruction));

    program_test.add_account(
        sender.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );
    program_test.add_account(
        receiver_1.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );
    program_test.add_account(
        receiver_2.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );   

    program_test.add_account(
        bank.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );         

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let (settings_pubkey, _) = Managment::get_settings_pubkey_with_bump(&program_id);
    let tx = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &MultisendInstruction::UpdateSettings { fee: 10 } ,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(settings_pubkey, false),
                AccountMeta::new(bank.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),                
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash
    );
    banks_client.process_transaction(tx).await.unwrap();



    let (settings_pubkey, _) = Managment::get_settings_pubkey_with_bump(&program_id);
    let tx = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &MultisendInstruction::UpdateSettings { fee: 20 } ,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(settings_pubkey, false),
                AccountMeta::new(bank.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash
    );
    banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &MultisendInstruction::SendLamports { amounts: vec![100000, 200000] },
            vec![
                AccountMeta::new(sender.pubkey(), true),
                AccountMeta::new(settings_pubkey, false),
                AccountMeta::new(bank.pubkey(), false),               
                AccountMeta::new(receiver_1.pubkey(), false),
                AccountMeta::new(receiver_2.pubkey(), false),
                AccountMeta::new(system_program::id(), false)
            ],
        )],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash
    );
    banks_client.process_transaction(tx).await.unwrap();

    let bank_account = banks_client
        .get_account(bank.pubkey())
        .await
        .expect("get_account")
        .expect("sender account not found");
    let bank_lamports = bank_account.lamports;

    let receiver_1_account = banks_client
        .get_account(receiver_1.pubkey())
        .await
        .expect("get_account")
        .expect("sender account not found");
    let receiver_1_lamports = receiver_1_account.lamports;

    let receiver_2_account = banks_client
        .get_account(receiver_2.pubkey())
        .await
        .expect("get_account")
        .expect("sender account not found");
    let receiver_2_lamports = receiver_2_account.lamports;

    assert_eq!(bank_lamports, 10_000_600);
    assert_eq!(receiver_1_lamports, 10_100_000);
    assert_eq!(receiver_2_lamports, 10_200_000);

}



#[tokio::test]
async fn test_transfer_lamports_update_bank() {
    
    let program_id = Pubkey::from_str("Transfer11111111111111111111111111111111111").unwrap();

    let sender = Keypair::new();
    let receiver_1 = Keypair::new();
    let receiver_2 = Keypair::new();
    let bank1 = Keypair::new();  
    let bank2 = Keypair::new();    

    let mut program_test = ProgramTest::new("multisend", program_id, processor!(process_instruction));

    program_test.add_account(
        sender.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );
    program_test.add_account(
        receiver_1.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );
    program_test.add_account(
        receiver_2.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );   

    program_test.add_account(
        bank1.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );    

 
    program_test.add_account(
        bank2.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );            

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let (settings_pubkey, _) = Managment::get_settings_pubkey_with_bump(&program_id);
    let tx = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &MultisendInstruction::UpdateSettings { fee: 10 } ,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(settings_pubkey, false),
                AccountMeta::new(bank1.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash
    );
    banks_client.process_transaction(tx).await.unwrap();



    let (settings_pubkey, _) = Managment::get_settings_pubkey_with_bump(&program_id);
    let tx = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &MultisendInstruction::UpdateSettings { fee: 10 } ,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(settings_pubkey, false),
                AccountMeta::new(bank2.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash
    );
    banks_client.process_transaction(tx).await.unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &MultisendInstruction::SendLamports { amounts: vec![100000, 200000] },
            vec![
                AccountMeta::new(sender.pubkey(), true),
                AccountMeta::new(settings_pubkey, false),
                AccountMeta::new(bank2.pubkey(), false),               
                AccountMeta::new(receiver_1.pubkey(), false),
                AccountMeta::new(receiver_2.pubkey(), false),
                AccountMeta::new(system_program::id(), false)
            ],
        )],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash
    );
    banks_client.process_transaction(tx).await.unwrap();

    let bank_account = banks_client
        .get_account(bank2.pubkey())
        .await
        .expect("get_account")
        .expect("sender account not found");
    let bank_lamports = bank_account.lamports;

    let receiver_1_account = banks_client
        .get_account(receiver_1.pubkey())
        .await
        .expect("get_account")
        .expect("sender account not found");
    let receiver_1_lamports = receiver_1_account.lamports;

    let receiver_2_account = banks_client
        .get_account(receiver_2.pubkey())
        .await
        .expect("get_account")
        .expect("sender account not found");
    let receiver_2_lamports = receiver_2_account.lamports;

    assert_eq!(bank_lamports, 10_000_300);
    assert_eq!(receiver_1_lamports, 10_100_000);
    assert_eq!(receiver_2_lamports, 10_200_000);

}



#[tokio::test]
async fn test_update_settings_not_admin() {
    
    let program_id = Pubkey::from_str("Transfer11111111111111111111111111111111111").unwrap();

    let sender = Keypair::new();
    let receiver_1 = Keypair::new();
    let receiver_2 = Keypair::new();
    let bank = Keypair::new();  
    let not_admin = Keypair::new();    

    let mut program_test = ProgramTest::new("multisend", program_id, processor!(process_instruction));

    program_test.add_account(
        sender.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );
    program_test.add_account(
        receiver_1.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );
    program_test.add_account(
        receiver_2.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );   

    program_test.add_account(
        bank.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );    

    program_test.add_account(
        not_admin.pubkey(),
        SolanaAccount {
            lamports: 10_000_000,
            ..SolanaAccount::default()
        },
    );        


    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let (settings_pubkey, _) = Managment::get_settings_pubkey_with_bump(&program_id);
    let tx = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &MultisendInstruction::UpdateSettings { fee: 10 } ,
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(settings_pubkey, false),
                AccountMeta::new(bank.pubkey(), false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash
    );
    banks_client.process_transaction(tx).await.unwrap();



    let (settings_pubkey, _) = Managment::get_settings_pubkey_with_bump(&program_id);
    let tx = Transaction::new_signed_with_payer(
        &[Instruction::new_with_borsh(
            program_id,
            &MultisendInstruction::UpdateSettings { fee: 20 } ,
            vec![
                AccountMeta::new(not_admin.pubkey(), true),
                AccountMeta::new(settings_pubkey, false),
                AccountMeta::new(bank.pubkey(), false),
                AccountMeta::new(not_admin.pubkey(), true),
                AccountMeta::new_readonly(sysvar::rent::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        )],
        Some(&not_admin.pubkey()),
        &[&not_admin],
        recent_blockhash
    );
    let result = banks_client.process_transaction(tx).await;
    assert!(result.is_err());
}





