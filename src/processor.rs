use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_instruction::create_account;
use solana_program::sysvar::Sysvar;
use crate::instruction::SocialInstruction;
use crate::state::UserProfile;

const PUB_KEY_SIZE: usize = 32;
const U16_SIZE: usize = 2;
const USER_PROFILE_SIZE: usize = 6;
const MAX_FOLLOWER_COUNT:usize = 200;


pub struct Processor;

impl Processor {

    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let social_instruction = SocialInstruction::try_from_slice(instruction_data)?;
        match social_instruction {
            SocialInstruction::Init(seed_type) => {
                Self::init(program_id, accounts, seed_type)
            }
            SocialInstruction::Follow(user) => {
                Ok(())
            }
            SocialInstruction::Unfollow(user) => {
                Ok(())
            }
            SocialInstruction::QueryFollows => {
                Ok(())
            }
            SocialInstruction::Post(user) => {
                Ok(())
            }
            SocialInstruction::QueryPosts => {
                Ok(())
            }
        }
    }

    fn init(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        seed_type: String
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_account = next_account_info(account_info_iter)?;
        let social_account = next_account_info(account_info_iter)?;
        let sys_program = next_account_info(account_info_iter)?;
        let seed = match seed_type.as_str() {
            "profile" => "profile",
            "post" => "post",
            _ => return Err(ProgramError::InvalidArgument)
        };
        msg!("seed: {:?}", seed);
        let (pda, bump) = Pubkey::find_program_address(&[user_account.key.as_ref(), seed.as_bytes()], program_id);
        msg!("pda: {:?}", pda);
        if pda != social_account.key.clone() {
            return Err(ProgramError::InvalidArgument);
        }
        let rent = Rent::get()?;
        let space = match seed_type.as_str() {
            "profile" => Self::cal_user_profile_size(MAX_FOLLOWER_COUNT),
            "post" => 0,
            _ => return Err(ProgramError::InvalidArgument)
        };
        let lamports = rent.minimum_balance(space);

        let create_account_ins = create_account(
            user_account.key,
            &social_account.key,
            lamports,
            space as u64,
            program_id,
        );
        let create_account_acc = [
            user_account.clone(),
            social_account.clone(),
            sys_program.clone()
        ];
        let create_account_sign = [[user_account.key.as_ref(), seed.as_bytes(), &[bump]].as_slice()].as_slice();
        invoke_signed(
            &create_account_ins,
            &create_account_acc,
            &create_account_sign
        )?;
        match seed_type.as_str() {
            "profile" => {
                let user_profile = UserProfile::new();
                user_profile.serialize(&mut *social_account.try_borrow_mut_data()?)?;
            }
            "post" => {},
            _ => return Err(ProgramError::InvalidArgument)
        };
        msg!("user profile init success");
    }

    fn cal_user_profile_size(pub_key_count: usize) -> usize {
        PUB_KEY_SIZE * pub_key_count + USER_PROFILE_SIZE
    }





}