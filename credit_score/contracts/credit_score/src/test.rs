#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{
    Env,
    testutils::{MockAuth, MockAuthInvoke, Address as _},
    IntoVal,
};

#[test]
fn test_set_and_get_score() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CreditScoreContract);
    let client = CreditScoreContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);

    client.mock_auths(&[MockAuth {
        address: &user,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "set_score",
            args: (user.clone(), 85u32).into_val(&env),
            sub_invokes: &[],
        },
    }]).set_score(&user, &85);

    // Solo validamos que la auth se hizo con la dirección correcta
    let auths = env.auths();
    assert_eq!(auths.len(), 1);
    assert_eq!(auths[0].0, user.clone());

    let score = client.get_score(&user);
    assert_eq!(score, 85);

    let other = Address::generate(&env);
    assert_eq!(client.get_score(&other), 0);
}

#[test]
#[should_panic(expected = "InvalidAction")]
fn test_set_score_unauthorized() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CreditScoreContract);
    let client = CreditScoreContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    client.set_score(&user, &70); // fallará con HostError(Auth,InvalidAction)
}


#[test]
fn test_get_score_without_setting() {
    let env = Env::default();
    let contract_id = env.register_contract(None, CreditScoreContract);
    let client = CreditScoreContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    assert_eq!(client.get_score(&user), 0);
}
