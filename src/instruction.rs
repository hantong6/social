use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub enum SocialInstruction {
    Init(String),
    Follow(Pubkey),
    Unfollow(Pubkey),
    QueryFollows,
    Post(String),
    QueryPosts
}