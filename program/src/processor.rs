#![allow(clippy::integer_arithmetic)]
//! Program instruction processor

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    program_error::ProgramError,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    sysvar::{rent::Rent, Sysvar},
    system_instruction,
    // msg
};

use spl_token::{self, state::Mint};

use borsh::{BorshDeserialize, BorshSerialize};

pub const SETTINGS_SEED: &str = "settings";

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Managment {
    pub fee: u64,
    pub admin: [u8; 32],
    pub bank: [u8; 32],
}

impl Managment {
    pub fn get_settings_pubkey_with_bump(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[SETTINGS_SEED.as_bytes()], program_id)
    }

}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum MultisendInstruction { 
    SendLamports { amounts: Vec<u64> }, 
    SendTokens { amounts: Vec<u64> }, 
    UpdateSettings { fee: u64 }
}


/// Instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    let instruction = MultisendInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        MultisendInstruction::SendLamports { amounts }=> {

            let acc_iter = &mut accounts.iter();

            let sender = next_account_info(acc_iter)?;            
            let settings = next_account_info(acc_iter)?;
            let bank = next_account_info(acc_iter)?;

            let settings_data = Managment::try_from_slice(&settings.data.borrow())?;
            if settings_data.bank != bank.key.to_bytes() && settings_data.bank != [0; 32] {
                return Err(ProgramError::Custom(1));
            }

            let total_lamports : u64 = amounts.iter().sum();
            let fee : u64 = ( total_lamports / 10000 ) * settings_data.fee;

            let ix = solana_program::system_instruction::transfer(sender.key, bank.key, fee);
            invoke(&ix, &[sender.clone(), bank.clone()])?;            

            for amount in amounts.into_iter() { 
                let receiver = next_account_info(acc_iter)?;
                let ix = solana_program::system_instruction::transfer(sender.key, receiver.key, amount);
                invoke(&ix, &[sender.clone(), receiver.clone()])?;
            }
        },
        MultisendInstruction::SendTokens { amounts }=> {
            let acc_iter = &mut accounts.iter();
            
            let sender = next_account_info(acc_iter)?;
            let settings = next_account_info(acc_iter)?;
            let bank = next_account_info(acc_iter)?;

            let token= next_account_info(acc_iter)?;
            let mint = next_account_info(acc_iter)?;
            let authority = next_account_info(acc_iter)?;

            let settings_data = Managment::try_from_slice(&settings.data.borrow())?;
            if settings_data.bank != bank.key.to_bytes() && settings_data.bank != [0; 32] {
                return Err(ProgramError::Custom(1));
            }

            let total_tokens : u64 = amounts.iter().sum();
            let fee : u64 = ( total_tokens / 10000 ) * settings_data.fee;


            let (expected_authority, bump_seed) = Pubkey::find_program_address(&[b"authority"], program_id);
            if expected_authority != *authority.key {
                return Err(ProgramError::InvalidSeeds);
            }
            
            let mint_data = Mint::unpack(&mint.try_borrow_data()?)?;
            let decimals = mint_data.decimals;

            let ix = spl_token::instruction::transfer_checked(
                token.key, 
                sender.key, 
                mint.key, 
                bank.key, 
                authority.key, 
                &[], 
                fee, 
                decimals
            )?;
            
            invoke_signed(
                &ix, 
                &[
                    sender.clone(),
                    mint.clone(),
                    bank.clone(),
                    authority.clone(),
                    token.clone(), // not required, but better for clarity
                ],
                &[&[b"authority", &[bump_seed]]],
            )?; 


            for amount in amounts.into_iter() { 
                let receiver = next_account_info(acc_iter)?;
                let ix = spl_token::instruction::transfer_checked(
                    token.key, 
                    sender.key, 
                    mint.key, 
                    receiver.key, 
                    authority.key, 
                    &[], 
                    amount, 
                    decimals
                )?;
                
                invoke_signed(
                    &ix, 
                    &[
                        sender.clone(),
                        mint.clone(),
                        receiver.clone(),
                        authority.clone(),
                        token.clone(), // not required, but better for clarity
                    ],
                    &[&[b"authority", &[bump_seed]]],
                )?;                
            }            
        },  
        MultisendInstruction::UpdateSettings { fee}=> {
            let acc_iter = &mut accounts.iter();

            let admin_info = next_account_info(acc_iter)?;
            let settings_info = next_account_info(acc_iter)?;
            let bank_info = next_account_info(acc_iter)?;
            let new_admin_info = next_account_info(acc_iter)?;
            let rent_info = next_account_info(acc_iter)?;
            let system_program_info = next_account_info(acc_iter)?;
    
            let (settings_pubkey, bump_seed) = Managment::get_settings_pubkey_with_bump(program_id);
            if settings_pubkey != *settings_info.key {
                return Err(ProgramError::InvalidArgument);
            }
    
            if !admin_info.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }
    
            if settings_info.data_is_empty() {
                let settings = Managment { fee, admin: new_admin_info.key.to_bytes(), bank: bank_info.key.to_bytes() };
                let space = settings.try_to_vec()?.len();
                let rent = &Rent::from_account_info(rent_info)?;
                let lamports = rent.minimum_balance(space);
                let signer_seeds: &[&[_]] = &[SETTINGS_SEED.as_bytes(), &[bump_seed]];
                invoke_signed(
                    &system_instruction::create_account(
                        admin_info.key,
                        &settings_pubkey,
                        lamports,
                        space as u64,
                        program_id,
                    ),
                    &[admin_info.clone(), settings_info.clone(), system_program_info.clone()],
                    &[&signer_seeds],
                )?;
            }
    
            let mut settings = Managment::try_from_slice(&settings_info.data.borrow())?;
            if settings.admin != admin_info.key.to_bytes() && settings.admin != [0; 32] {
                return Err(ProgramError::Custom(1));
            }
            settings.fee = fee;
            settings.admin = new_admin_info.key.to_bytes();
            settings.bank = bank_info.key.to_bytes();
            settings.serialize(&mut &mut settings_info.data.borrow_mut()[..])?;
        }      
    }

    Ok(())
}