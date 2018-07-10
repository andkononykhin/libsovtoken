#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate rust_indy_sdk as indy;
extern crate sovtoken;

mod utils;

use std::collections::HashMap;
use indy::ErrorCode;

pub const ATTRIB_RAW_DATA_2: &'static str = r#"{"endpoint":{"ha":"127.0.0.1:5555"}}"#;
pub const ATTRIB_RAW_DATA: &'static str = r#"{"endpoint":{"ha":"127.0.0.1:5555"}}"#;

#[test]
pub fn build_and_submit_attrib_with_fees() {
    sovtoken::api::sovtoken_init();
    let payment_method = sovtoken::api::PAYMENT_METHOD_NAME;
    let pc_str = utils::pool::create_pool_config();
    let pool_config = Some(pc_str.as_str());
    indy::pool::Pool::set_protocol_version(2).unwrap();

    let pool_name = utils::pool::create_pool_ledger(pool_config);
    let pool_handle = indy::pool::Pool::open_ledger(&pool_name, None).unwrap();
    let wallet = utils::wallet::Wallet::new();

    let trustees = utils::did::initial_trustees(4, wallet.handle, pool_handle).unwrap();
    let dids = utils::did::did_str_from_trustees(&trustees);

    let pa1 = utils::payment::address::generate(&wallet, None);

    let mut mint_cfg = HashMap::new();
    mint_cfg.insert(pa1.clone(), 10);

    utils::mint::mint_tokens(mint_cfg, pool_handle, wallet.handle, &dids).unwrap();

    let (utxo, _, _) = utils::get_utxo::get_first_utxo_for_payment_address(wallet.handle, pool_handle, dids[0], &pa1);

    let inputs = json!([utxo]).to_string();
    let outputs = json!([{
        "paymentAddress": pa1,
        "amount": 9
    }]).to_string();

    let fees = json!({
        "100": 1
    }).to_string();

    utils::fees::set_fees(pool_handle, wallet.handle, payment_method, &fees, &dids);

    let parsed_resp = _send_attrib_with_fees(dids[0], Some(ATTRIB_RAW_DATA), wallet.handle, pool_handle, &inputs, &outputs).unwrap();

    let parsed_resp_json: Vec<HashMap<String, serde_json::Value>> = serde_json::from_str(&parsed_resp).unwrap();
    assert_eq!(parsed_resp_json.len(), 1);
    assert_eq!(parsed_resp_json[0].get("amount").unwrap().as_u64().unwrap(), 9);
    assert_eq!(parsed_resp_json[0].get("paymentAddress").unwrap().as_str().unwrap(), pa1);

    let get_attrib_req = indy::ledger::Ledger::build_get_attrib_request(dids[0], dids[0], Some("endpoint"), None, None).unwrap();
    let get_attrib_resp = indy::ledger::Ledger::sign_and_submit_request(pool_handle, wallet.handle, dids[0], &get_attrib_req).unwrap();
    let get_attrib_resp_json: serde_json::Value = serde_json::from_str(&get_attrib_resp).unwrap();
    assert_eq!(ATTRIB_RAW_DATA, get_attrib_resp_json.as_object().unwrap().get("result").unwrap().as_object().unwrap().get("data").unwrap().as_str().unwrap());

    let fees = json!({
        "100": 0
    }).to_string();

    utils::fees::set_fees(pool_handle, wallet.handle, payment_method, &fees, &dids);
}

#[test]
#[ignore]
pub fn build_and_submit_attrib_with_fees_insufficient_funds() {
    sovtoken::api::sovtoken_init();
    let payment_method = sovtoken::api::PAYMENT_METHOD_NAME;
    let pc_str = utils::pool::create_pool_config();
    let pool_config = Some(pc_str.as_str());
    indy::pool::Pool::set_protocol_version(2).unwrap();

    let pool_name = utils::pool::create_pool_ledger(pool_config);
    let pool_handle = indy::pool::Pool::open_ledger(&pool_name, None).unwrap();
    let wallet = utils::wallet::Wallet::new();

    let trustees = utils::did::initial_trustees(4, wallet.handle, pool_handle).unwrap();
    let dids = utils::did::did_str_from_trustees(&trustees);

    let pa1 = utils::payment::address::generate(&wallet, None);

    let mut mint_cfg = HashMap::new();
    mint_cfg.insert(pa1.clone(), 9);

    utils::mint::mint_tokens(mint_cfg, pool_handle, wallet.handle, &dids).unwrap();

    let (utxo, _, _) = utils::get_utxo::get_first_utxo_for_payment_address(wallet.handle, pool_handle, dids[0], &pa1);

    let inputs = json!([utxo]).to_string();
    let outputs = json!([{
        "paymentAddress": pa1,
        "amount": 9
    }]).to_string();

    let fees = json!({
        "100": 1
    }).to_string();

    utils::fees::set_fees(pool_handle, wallet.handle, payment_method, &fees, &dids);

    let parsed_err = _send_attrib_with_fees(dids[0], Some(ATTRIB_RAW_DATA), wallet.handle, pool_handle, &inputs, &outputs).unwrap_err();
    assert_eq!(parsed_err, ErrorCode::PaymentInsufficientFundsError);

    let fees = json!({
        "100": 0
    }).to_string();

    utils::fees::set_fees(pool_handle, wallet.handle, payment_method, &fees, &dids);
}

#[test]
#[ignore]
pub fn build_and_submit_attrib_with_fees_double_spend() {
    sovtoken::api::sovtoken_init();
    let payment_method = sovtoken::api::PAYMENT_METHOD_NAME;
    let pc_str = utils::pool::create_pool_config();
    let pool_config = Some(pc_str.as_str());
    indy::pool::Pool::set_protocol_version(2).unwrap();

    let pool_name = utils::pool::create_pool_ledger(pool_config);
    let pool_handle = indy::pool::Pool::open_ledger(&pool_name, None).unwrap();
    let wallet = utils::wallet::Wallet::new();

    let trustees = utils::did::initial_trustees(4, wallet.handle, pool_handle).unwrap();
    let dids = utils::did::did_str_from_trustees(&trustees);

    let pa1 = utils::payment::address::generate(&wallet, None);

    let mut mint_cfg = HashMap::new();
    mint_cfg.insert(pa1.clone(), 10);

    utils::mint::mint_tokens(mint_cfg, pool_handle, wallet.handle, &dids).unwrap();

    let (utxo, _, _) = utils::get_utxo::get_first_utxo_for_payment_address(wallet.handle, pool_handle, dids[0], &pa1);

    let inputs = json!([utxo]).to_string();
    let outputs = json!([{
        "paymentAddress": pa1,
        "amount": 9
    }]).to_string();

    let fees = json!({
        "100": 1
    }).to_string();

    utils::fees::set_fees(pool_handle, wallet.handle, payment_method, &fees, &dids);

    let parsed_resp = _send_attrib_with_fees(dids[0], Some(ATTRIB_RAW_DATA), wallet.handle, pool_handle, &inputs, &outputs).unwrap();

    let parsed_resp_json: Vec<HashMap<String, serde_json::Value>> = serde_json::from_str(&parsed_resp).unwrap();
    assert_eq!(parsed_resp_json.len(), 1);
    assert_eq!(parsed_resp_json[0].get("amount").unwrap().as_u64().unwrap(), 9);
    assert_eq!(parsed_resp_json[0].get("paymentAddress").unwrap().as_str().unwrap(), pa1);

    let get_attrib_req = indy::ledger::Ledger::build_get_attrib_request(dids[0], dids[0], Some("endpoint"), None, None).unwrap();
    let get_attrib_resp = indy::ledger::Ledger::sign_and_submit_request(pool_handle, wallet.handle, dids[0], &get_attrib_req).unwrap();
    let get_attrib_resp_json: serde_json::Value = serde_json::from_str(&get_attrib_resp).unwrap();
    assert_eq!(ATTRIB_RAW_DATA, get_attrib_resp_json.as_object().unwrap().get("result").unwrap().as_object().unwrap().get("data").unwrap().as_str().unwrap());

    let _parsed_err = _send_attrib_with_fees(dids[0], Some(ATTRIB_RAW_DATA_2), wallet.handle, pool_handle, &inputs, &outputs).unwrap_err();
    //assert_eq!(parsed_err, ErrorCode::PaymentUTXODoesNotExist);
    //TODO: this test should fail for awhile until we get some vision on a ErrorCodes (both on parsing and new ones)
    assert!(false);

    let fees = json!({
        "100": 0
    }).to_string();

    utils::fees::set_fees(pool_handle, wallet.handle, payment_method, &fees, &dids);
}

fn _send_attrib_with_fees(did: &str, data: Option<&str>, wallet_handle: i32, pool_handle: i32, inputs: &str, outputs: &str) -> Result<String, ErrorCode> {
    let attrib_req = indy::ledger::Ledger::build_attrib_request(did, did,  None, data, None).unwrap();
    let attrib_req_signed = indy::ledger::Ledger::sign_request(wallet_handle, did, &attrib_req).unwrap();
    let (attrib_req_with_fees, pm) = indy::payments::Payment::add_request_fees(wallet_handle, did, &attrib_req_signed, inputs, outputs).unwrap();
    let attrib_resp = indy::ledger::Ledger::submit_request(pool_handle, &attrib_req_with_fees).unwrap();
    indy::payments::Payment::parse_response_with_fees(&pm, &attrib_resp)
}