#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    Env, Address, symbol_short, Vec,
};
use soroban_sdk::token;

// 📄 Estructura de oferta
#[contracttype]
#[derive(Clone)]
pub struct LenderOffer {
    pub lender: Address,
    pub amount: i128,
    pub rate_bps: u32,
    pub timestamp: u64,
    pub active: bool,
}

// 🔑 DataKey para storage
#[contracttype]
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum DataKey {
    NextOfferId,
    Offer(u64),
}

#[contract]
pub struct LendingP2P;

#[contractimpl]
impl LendingP2P {
    fn gen_offer_id(env: &Env) -> u64 {
        let id: u64 = env.storage().persistent().get(&DataKey::NextOfferId).unwrap_or(0);
        env.storage().persistent().set(&DataKey::NextOfferId, &(id + 1));
        id
    }

    pub fn create_offer(env: Env, lender: Address, amount: i128, rate_bps: u32) -> u64 {
        lender.require_auth();
        let usdc = load_usdc(&env);
        usdc.transfer(&lender, &env.current_contract_address(), &amount);

        let id = Self::gen_offer_id(&env);
        let offer = LenderOffer {
            lender: lender.clone(),
            amount,
            rate_bps,
            timestamp: env.ledger().timestamp(),
            active: true,
        };
        env.storage().persistent().set(&DataKey::Offer(id), &offer);

        // ⚠️ símbolos ≤ 9 caracteres para symbol_short!
        env.events().publish(
            (symbol_short!("LEND"), symbol_short!("O_CRT")), 
            (id, lender, amount, rate_bps),
        );
        id
    }

    pub fn cancel_offer(env: Env, lender: Address, offer_id: u64) {
        lender.require_auth();
        let mut off: LenderOffer = env.storage().persistent().get(&DataKey::Offer(offer_id))
            .expect("offer not found");
        assert!(off.active && off.lender == lender, "No autorizado o ya inactiva");
        off.active = false;
        env.storage().persistent().set(&DataKey::Offer(offer_id), &off);

        let usdc = load_usdc(&env);
        usdc.transfer(&env.current_contract_address(), &lender, &off.amount);

        env.events().publish(
            (symbol_short!("LEND"), symbol_short!("O_CAN")), 
            offer_id,
        );
    }

    pub fn take_offer(env: Env, borrower: Address, offer_id: u64, collateral: i128) {
        borrower.require_auth();
        let mut off: LenderOffer = env.storage().persistent().get(&DataKey::Offer(offer_id))
            .expect("offer not found");
        assert!(off.active, "Oferta no disponible");
        off.active = false;
        env.storage().persistent().set(&DataKey::Offer(offer_id), &off);

        let xlm = load_xlm(&env);
        xlm.transfer(&borrower, &env.current_contract_address(), &collateral);
        let usdc = load_usdc(&env);
        usdc.transfer(&env.current_contract_address(), &borrower, &off.amount);

        env.events().publish(
            (symbol_short!("LEND"), symbol_short!("O_TKN")), 
            (offer_id, borrower, collateral),
        );
    }

    // Especificar tipos en get<_, LenderOffer>()
    pub fn list_offers(env: Env) -> Vec<(u64, LenderOffer)> {
        let max = env.storage().persistent().get(&DataKey::NextOfferId).unwrap_or(0);
        let mut v: Vec<(u64, LenderOffer)> = Vec::new(&env);
        for id in 0..max {
            if let Some(off) = env.storage().persistent().get::<DataKey, LenderOffer>(&DataKey::Offer(id)) {
                if off.active {
                    v.push_back((id, off));
                }
            }
        }
        v
    }
    
}

fn load_usdc(e: &Env) -> token::Client {
    let id: Address = e.storage().persistent().get(&symbol_short!("USDC"))
        .expect("USDC token id not set");
    token::Client::new(e, &id)
}

fn load_xlm(e: &Env) -> token::Client {
    let id: Address = e.storage().persistent().get(&symbol_short!("XLM"))
        .expect("XLM token id not set");
    token::Client::new(e, &id)
}
 
mod test;