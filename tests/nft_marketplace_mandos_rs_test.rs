use elrond_wasm::types::{
    BigUint, EsdtLocalRole, EsdtTokenPayment, ManagedAddress, SCResult, TokenIdentifier,
};
use elrond_wasm_debug::{
    assert_sc_error, managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::*, tx_mock::TxInputESDT,
};
use rust_testing_framework_tester::*;

const TEST_OUTPUT_PATH: &'static str = "test.scen.json";
const TEST_MULTIPLE_SC_OUTPUT_PATH: &'static str = "test_multiple_sc.scen.json";
const TEST_ESDT_OUTPUT_PATH: &'static str = "test_esdt_generation.scen.json";

const SC_WASM_PATH: &'static str = "output/rust-testing-framework-tester.wasm";
const ADDER_WASM_PATH: &'static str = "../../examples/adder/output/adder.wasm";

#[test]
fn test_query() {
    let mut wrapper = BlockchainStateWrapper::new();
    let sc_wrapper = wrapper.create_sc_account(
        &rust_biguint!(2_000),
        None,
        rust_testing_framework_tester::contract_obj,
        SC_WASM_PATH,
    );

    let _ = wrapper.execute_query(&sc_wrapper, |sc| {
        let actual_balance = sc.get_egld_balance();
        let expected_balance = managed_biguint!(2_000);
        assert_eq!(actual_balance, expected_balance);
    });
}
