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
    pub time: u64
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

    pub fn unfollow(&mut self, follower: Pubkey) {
        self.followers.retain(|&x| x != follower);
        self.data_len = self.followers.len() as u16;
    }

}

impl UserPost {

    pub fn new() -> Self {
        UserPost {
            post_count: 0
        }
    }

    pub fn add_post(&mut self) {
        self.post_count += 1;
    }

    pub fn get_count(&self) -> u64 {
        self.post_count
    }

}

impl Post {

    pub fn new(content: String, time: u64) -> Self {
        Post {
            content,
            time
        }
    }

}