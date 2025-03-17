use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct UserProfile {
    pub data_len: u16,
    pub followers: Vec<Pubkey>
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct UserPost {
    pub post_count: u64
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Post {
    pub content: String,
    pub timestamp: u64
}

impl UserProfile {

    pub fn new() -> Self {
        UserProfile {
            data_len: 0,
            followers: vec![]
        }
    }

    pub fn follow(&mut self, follower: Pubkey) {
        self.followers.push(follower);
        self.data_len = self.followers.len() as u16;
    }

}