use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::{next_account_info, AccountInfo};
use solana_program::borsh1::try_from_slice_unchecked;
use solana_program::clock::Clock;
use solana_program::entrypoint::ProgramResult;
use solana_program::msg;
use solana_program::program::invoke_signed;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::system_instruction::create_account;
use solana_program::sysvar::Sysvar;
use crate::instruction::SocialInstruction;
use crate::state::{Post, UserPost, UserProfile};

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
        msg!("social instruction: {:?}", social_instruction);
        match social_instruction {
            SocialInstruction::Init(seed_type) => {
                Self::init(program_id, accounts, seed_type)
            }
            SocialInstruction::Follow(user) => {
                Self::follow(accounts, user)
            }
            SocialInstruction::Unfollow(user) => {
                Self::unfollow(accounts, user)
            }
            SocialInstruction::QueryFollows => {
                Self::query_followers(accounts)
            }
            SocialInstruction::Post(content) => {
                Self::post(program_id, accounts, content)
            }
            SocialInstruction::QueryPosts => {
                Self::query_post(accounts)
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
        if pda != *social_account.key {
            return Err(ProgramError::InvalidArgument);
        }
        let rent = Rent::get()?;
        let space = match seed_type.as_str() {
            "profile" => Self::cal_user_profile_size(MAX_FOLLOWER_COUNT),
            "post" => 8,
            _ => return Err(ProgramError::InvalidArgument)
        };
        let lamports = rent.minimum_balance(space);

        let create_account_ins = create_account(
            user_account.key,
            social_account.key,
            lamports,
            space as u64,
            program_id,
        );
        let create_account_acc = [
            user_account.clone(),
            social_account.clone(),
            sys_program.clone()
        ];
        invoke_signed(
            &create_account_ins,
            &create_account_acc,
            &[&[user_account.key.as_ref(), seed.as_bytes(), &[bump]]]
        )?;
        match seed_type.as_str() {
            "profile" => {
                let user_profile = UserProfile::new();
                user_profile.serialize(&mut *social_account.try_borrow_mut_data()?)?;
            }
            "post" => {
                let user_post = UserPost::new();
                user_post.serialize(&mut *social_account.data.borrow_mut())?;
            },
            _ => return Err(ProgramError::InvalidArgument)
        };
        msg!("user profile init success");
        Ok(())
    }

    fn follow(
        accounts: &[AccountInfo],
        user: Pubkey
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let social_account = next_account_info(account_info_iter)?;
        let mut size: usize = 0;
        {
            let data = &social_account.data.borrow();
            let len = &data[..U16_SIZE];
            let followers_count = Self::cal_bytes_to_u16(len).unwrap();
            size = Self::cal_user_profile_size(followers_count as usize);
        }
        let mut user_profile = UserProfile::try_from_slice(&social_account.data.borrow()[..size])?;
        msg!("current user profile: {:?}", user_profile);
        user_profile.follow(user);
        user_profile.serialize(&mut *social_account.data.borrow_mut())?;
        Ok(())
    }

    fn query_followers(
        accounts: &[AccountInfo]
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let social_account = next_account_info(account_info_iter)?;
        let user_profile = try_from_slice_unchecked::<UserProfile>(*social_account.data.borrow())?;
        msg!("current user profile: {:?}", user_profile);
        Ok(())
    }

    fn unfollow(
        accounts: &[AccountInfo],
        user: Pubkey
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let social_account = next_account_info(account_info_iter)?;
        let mut user_profile = try_from_slice_unchecked::<UserProfile>(*social_account.data.borrow())?;
        user_profile.unfollow(user);
        user_profile.serialize(&mut *social_account.data.borrow_mut())?;
        msg!("current user profile: {:?}", user_profile);
        Ok(())
    }

    fn post(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        content: String
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_account = next_account_info(account_info_iter)?;
        let social_account = next_account_info(account_info_iter)?;
        let social_post_account = next_account_info(account_info_iter)?;
        let sys_program = next_account_info(account_info_iter)?;

        let clock = Clock::get()?;
        let time = clock.unix_timestamp as u64;
        //post metadata
        let mut user_post = try_from_slice_unchecked::<UserPost>(&social_account.data.borrow())?;
        user_post.add_post();
        user_post.serialize(&mut *social_account.data.borrow_mut())?;
        let count = user_post.get_count();
        //post pda
        let (post_pda, bump) = Pubkey::find_program_address(&[user_account.key.as_ref(), "post".as_bytes(), &[count as u8]], program_id);
        msg!("post pda: {:?}", post_pda);
        if post_pda != *social_post_account.key {
            return Err(ProgramError::InvalidArgument);
        }
        let post = Post::new(content, time);
        let rent = Rent::get()?;
        let space = borsh::to_vec(&post)?.len();
        let lamports = rent.minimum_balance(space);

        let create_account_ins = create_account(
            user_account.key,
            social_post_account.key,
            lamports,
            space as u64,
            program_id,
        );
        let create_account_acc = [
            user_account.clone(),
            social_post_account.clone(),
            sys_program.clone()
        ];
        invoke_signed(
            &create_account_ins,
            &create_account_acc,
            &[&[user_account.key.as_ref(), "post".as_bytes(), &[bump]]]
        )?;
        //save account
        post.serialize(&mut *social_post_account.data.borrow_mut())?;
        msg!("user post success");
        Ok(())
    }

    fn query_post(
        accounts: &[AccountInfo]
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let social_account = next_account_info(account_info_iter)?;
        let social_post_account = next_account_info(account_info_iter)?;
        let user_post = try_from_slice_unchecked::<UserPost>(&social_account.data.borrow())?;
        msg!("current user post: {:?}", user_post);
        let post = try_from_slice_unchecked::<Post>(&social_post_account.data.borrow())?;
        msg!("current post: {:?}", post);
        Ok(())
    }

    fn cal_user_profile_size(pub_key_count: usize) -> usize {
        PUB_KEY_SIZE * pub_key_count + USER_PROFILE_SIZE
    }

    fn cal_bytes_to_u16(bytes: &[u8]) -> Option<u16> {
        if bytes.len() != 2 {
            return None
        }
        let mut array = [0u8; 2];
        array.copy_from_slice(bytes);
        Some(u16::from_be_bytes(array))
    }

}