#![cfg(test)]
extern crate std;

use soroban_sdk::{Env, symbol_short, testutils::Events, Address};
use super::*;

// Importa spec de token WASM si deseas interacción real:
// soroban_sdk::contractimport!(file = "../path/to/soroban_token_spec.wasm");

#[test]
fn test_flows() {
    let env = Env::default();

    // 🚀 Registra tu contrato
    let contract_id = env.register(LendingP2P, ());
    let client = LendingP2PClient::new(&env, &contract_id);

    // 🎯 Registra dos tokens placeholder (simulación)
    let usdc_token = env.register_dummy_contract();
    let xlm_token = env.register_dummy_contract();
    env.storage().persistent().set(&symbol_short!("USDC"), &usdc_token);
    env.storage().persistent().set(&symbol_short!("XLM"), &xlm_token);

    // 🤖 Crea direcciones ficticias
    let lender = Address::random(&env);
    let borrower = Address::random(&env);

    // 🧾 Prepara saldos (dummy tokens permiten transferencias sin error)
    let amount = 1000i128;
    let rate = 500u32;

    // 1️⃣ Crear oferta
    let offer_id = client.create_offer(&lender, &amount, &rate);
    assert_eq!(offer_id, 0);

    // 2️⃣ Verifica que se emitió evento OFFER_CREATED
    let events = env.events();
    let all = events.all();
    assert!(all.iter().any(|(_, topics, _)| topics.contains(&symbol_short!("O_CRT"))));

    // 3️⃣ Revisa storage: oferta creada y activa
    let stored = env.storage().persistent().get::<DataKey, LenderOffer>(&DataKey::Offer(0)).unwrap();
    assert_eq!(stored.amount, amount);
    assert!(stored.active);

    // 4️⃣ Cancelar oferta
    client.cancel_offer(&lender, &0);
    let stored2 = env.storage().persistent().get::<DataKey, LenderOffer>(&DataKey::Offer(0)).unwrap();
    assert!(!stored2.active);
    let all2 = env.events().all();
    assert!(all2.iter().any(|(_, topics, _)| topics.contains(&symbol_short!("O_CAN"))));

    // 5️⃣ Nueva oferta y tomarla
    let offer_id2 = client.create_offer(&lender, &200, &700);
    client.take_offer(&borrower, &offer_id2, &50);
    let stored3 = env.storage().persistent().get::<DataKey, LenderOffer>(&DataKey::Offer(offer_id2)).unwrap();
    assert!(!stored3.active);
    let all3 = env.events().all();
    assert!(all3.iter().any(|(_, topics, _)| topics.contains(&symbol_short!("O_TKN"))));
}
