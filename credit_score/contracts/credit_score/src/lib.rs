#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Symbol};

const SCORE_KEY: Symbol = symbol_short!("SCORE");

#[contract]
pub struct CreditScoreContract;

#[contractimpl]
impl CreditScoreContract {
    pub fn set_score(env: Env, user: Address, score: u32) {
        user.require_auth();
        let key = (SCORE_KEY, user.clone());
        env.storage().instance().set(&key, &score);
    }

    pub fn get_score(env: Env, user: Address) -> u32 {
        let key = (SCORE_KEY, user.clone());
        env.storage()
           .instance()
           .get(&key)
           .unwrap_or(0)
    }
}
mod test;