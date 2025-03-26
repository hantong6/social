mod instruction;
mod processor;
mod state;
mod error;

use solana_program::account_info::AccountInfo;
use solana_program::{entrypoint, msg};
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use crate::processor::Processor;

entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    Processor::process_instruction(program_id, _accounts, _instruction_data)
}