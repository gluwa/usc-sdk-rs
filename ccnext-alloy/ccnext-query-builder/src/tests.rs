use ccnext_abi_encoding::abi::abi_encode;
use crate::{
    {abi::{models::QueryableFields, query_builder::QueryBuilder}, test_helpers::{
        check_results, get_transaction_and_receipt, get_vrs, get_y_parity, ResultField
    }},
};
use alloy::consensus::Transaction;
use std::sync::Arc;

// Tx/Rx Fields queried in this test: All legacy (type 0) fields except Tx Data
// See abi_encoding_mapping.rs for details
#[tokio::test]
async fn legacy_tx_queried_fields_match_expected() {
    // Get legacy transaction via rpc
    let (tx, rx) = get_transaction_and_receipt(
        "0xc990ce703dd3ca83429c302118f197651678de359c271f205b9083d4aa333aae",
    )
    .await;
    assert!(tx.inner.is_legacy());

    // Encode transaction
    let encoded = abi_encode(tx.clone(), rx.clone()).unwrap();

    let mut query_builder = QueryBuilder::create_from_transaction(tx.clone(), rx.clone())
        .expect("creating queryable builder should work");
    query_builder.set_abi_provider(Arc::new(|contract_address| {
        Box::pin(test_abi_provider(contract_address.clone()))
    }));

    query_builder
        .add_static_field(QueryableFields::Type)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::TxNonce)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::TxGasPrice)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::TxGasLimit)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::TxFrom)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::TxTo)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::TxValue)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::TxV)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::TxR)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::TxS)
        .unwrap();

    let selected_offsets = query_builder.get_selected_offsets();
    let raw = encoded.abi.clone();

    let (v, r, s) = get_vrs(&tx);

    let expected_results: Vec<ResultField> = vec![
        ResultField::TxType(tx.inner.tx_type().into()),
        ResultField::TxNonce(tx.nonce()),
        ResultField::TxGasPrice(tx.gas_price().expect("Legacy tx should have this")),
        ResultField::TxGasLimit(tx.gas_limit()),
        ResultField::EthAddress(tx.from), // Caller address
        ResultField::EthAddress(tx.to().expect("Should be to field in contract call")), // Contract address in call
        ResultField::TxValue(tx.value()),
        ResultField::TxV(v),
        ResultField::TxR(r),
        ResultField::TxS(s),
    ];

    // Checking that all result data matches expected
    check_results(expected_results, selected_offsets, raw.clone());
}

// Tx/Rx Fields queried in this test:
// - Rx Status
// - Rx Gas Used
// - Rx Log Blooms
#[tokio::test]
async fn queried_receipt_fields_match_expected() {
    // Get legacy transaction via rpc
    let (tx, rx) = get_transaction_and_receipt(
        "0xc990ce703dd3ca83429c302118f197651678de359c271f205b9083d4aa333aae",
    )
    .await;
    assert!(tx.inner.is_legacy());

    // Encode transaction
    let encoded = abi_encode(tx.clone(), rx.clone()).unwrap();

    let mut query_builder = QueryBuilder::create_from_transaction(tx.clone(), rx.clone())
        .expect("creating queryable builder should work");
    query_builder.set_abi_provider(Arc::new(|contract_address| {
        Box::pin(test_abi_provider(contract_address.clone()))
    }));

    query_builder
        .add_static_field(QueryableFields::RxStatus)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::RxGasUsed)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::RxLogBlooms)
        .unwrap();

    let selected_offsets = query_builder.get_selected_offsets();
    let raw = encoded.abi.clone();
    let expected_results: Vec<ResultField> = vec![
        ResultField::RxStatus(rx.status() as u8),
        ResultField::RxGasUsed(rx.gas_used),
        ResultField::RxLogBlooms(rx.inner.logs_bloom().0.to_vec()),
    ];

    // Checking that all result data matches expected
    check_results(expected_results, selected_offsets, raw.clone());
}

// Tx/Rx Fields queried in this test:
// - Log 1 (Transfer event):
//     - Event addr (contract addr)
//     - Event index 0 (signature)
//     - Event index 1 (from address)
//     - Event index 2 (to, burn address)
//     - Event data field 0 (burned amount)
#[tokio::test]
async fn event_builder_queried_fields_match_expected() {
    // Get legacy transaction via rpc
    let (tx, rx) = get_transaction_and_receipt(
        "0xc990ce703dd3ca83429c302118f197651678de359c271f205b9083d4aa333aae",
    )
    .await;
    assert!(tx.inner.is_legacy());

    // Encode transaction
    let encoded = abi_encode(tx.clone(), rx.clone()).unwrap();

    let mut query_builder = QueryBuilder::create_from_transaction(tx.clone(), rx.clone())
        .expect("creating queryable builder should work");
    query_builder.set_abi_provider(Arc::new(|contract_address| {
        Box::pin(test_abi_provider(contract_address.clone()))
    }));

    // just to keep it simple for now
    // i'll just say i care about the event index 1
    // which is the transfer event from, to (zero address), amount
    query_builder
        .event_builder(
            "Transfer".into(),
            |_log, _event, log_index| log_index == 1,
            |builder| {
                // fun no? :)
                builder
                    .add_address()
                    .unwrap()
                    .add_signature()
                    .unwrap()
                    .add_argument("from")
                    .unwrap()
                    .add_argument("to")
                    .unwrap()
                    .add_argument("value")
                    .unwrap();

                Ok(())
            },
        )
        .await
        .expect("should have matched an event and constructed offsets");

    let selected_offsets = query_builder.get_selected_offsets();
    let raw = encoded.abi.clone();

    let expected_results: Vec<ResultField> = vec![
        ResultField::EthAddress(rx.inner.logs()[1].address()), // Contract address in event
        ResultField::EventTopic(rx.inner.logs()[1].topic0().unwrap().0), // Event signature
        ResultField::EventTopic(rx.inner.logs()[1].topics()[1].0), // Event from address
        ResultField::EventTopic(rx.inner.logs()[1].topics()[2].0), // Event to address
        ResultField::EventDataField(
            rx.inner.logs()[1].data().data[..]
                .try_into()
                .expect("Data should contain 1 32 byte field for this transaction"),
        ), // Event value
    ];

    // Checking that all result data matches expected
    check_results(expected_results, selected_offsets, raw.clone());
}

// Tx/Rx Fields queried in this test:
// - Call Data:
//     - Function signature
//     - data field 0 (burned amount)
#[tokio::test]
async fn function_builder_queried_fields_match_expected() {
    // Get legacy transaction via rpc
    let (tx, rx) = get_transaction_and_receipt(
        "0xc990ce703dd3ca83429c302118f197651678de359c271f205b9083d4aa333aae",
    )
    .await;
    assert!(tx.inner.is_legacy());

    // Encode transaction
    let encoded = abi_encode(tx.clone(), rx.clone()).unwrap();

    let mut query_builder = QueryBuilder::create_from_transaction(tx.clone(), rx.clone())
        .expect("creating queryable builder should work");
    query_builder.set_abi_provider(Arc::new(|contract_address| {
        Box::pin(test_abi_provider(contract_address.clone()))
    }));

    query_builder
        .function_builder("burn".into(), |b| {
            b.add_signature()
                .unwrap()
                .add_argument("value".into())
                .unwrap();
            Ok(())
        })
        .await
        .expect("should be able to query some calldata segments");

    let selected_offsets = query_builder.get_selected_offsets();
    let raw = encoded.abi.clone();

    let expected_results: Vec<ResultField> = vec![
        ResultField::FunctionSignifier(
            tx.inner.as_legacy().unwrap().tx().input[0..4]
                .try_into()
                .expect("4 bytes is right slice length"),
        ),
        ResultField::CallDataField(
            tx.inner.as_legacy().unwrap().tx().input[4..36]
                .try_into()
                .expect("32 bytes is right len"),
        ), // Burn value in function call
    ];

    // Checking that all result data matches expected
    check_results(expected_results, selected_offsets, raw.clone());
}

// Tx/Rx Fields queried in this test: All fields newly added in type 1
// - Tx Chain Id
// - Tx y parity
#[tokio::test]
async fn type_1_tx_queried_fields_match_expected() {
    // Get legacy transaction via rpc
    let (tx, rx) = get_transaction_and_receipt(
        "0x5c8c6d8c61bd8109ce02717db62b12554c097d156b66e30ff64864b5d4b1c041",
    )
    .await;
    assert!(tx.inner.is_eip2930());

    // Encode transaction
    let encoded = abi_encode(tx.clone(), rx.clone()).unwrap();

    let mut query_builder = QueryBuilder::create_from_transaction(tx.clone(), rx.clone())
        .expect("creating queryable builder should work");
    query_builder.set_abi_provider(Arc::new(|contract_address| {
        Box::pin(test_abi_provider(contract_address.clone()))
    }));

    query_builder
        .add_static_field(QueryableFields::TxChainId)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::TxYParity)
        .unwrap();

    let selected_offsets = query_builder.get_selected_offsets();
    let raw = encoded.abi.clone();

    let expected_results: Vec<ResultField> = vec![
        ResultField::TxChainId(tx.chain_id().unwrap()),
        ResultField::TxYParity(get_y_parity(&tx))
    ];

    // Checking that all result data matches expected
    check_results(expected_results, selected_offsets, raw.clone());
}

// Tx/Rx Fields queried in this test: All fields newly added in type 2
// - TxMaxPriorityFeePerGas
// - TxMaxFeePerGas
// - TxAccessList
#[tokio::test]
async fn type_2_tx_queried_fields_match_expected() {
    // Get legacy transaction via rpc
    let (tx, rx) = get_transaction_and_receipt(
        "0xdfba59b94bac3da5af5d0fa8b81ae3199069fa6f38002be58c14e94a051e0642",
    )
    .await;
    assert!(tx.inner.is_eip1559());

    // Encode transaction
    let encoded = abi_encode(tx.clone(), rx.clone()).unwrap();

    let mut query_builder = QueryBuilder::create_from_transaction(tx.clone(), rx.clone())
        .expect("creating queryable builder should work");
    query_builder.set_abi_provider(Arc::new(|contract_address| {
        Box::pin(test_abi_provider(contract_address.clone()))
    }));

    query_builder
        .add_static_field(QueryableFields::TxMaxPriorityFeePerGas)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::TxMaxFeePerGas)
        .unwrap();
    //query_builder
    //    .add_static_field(QueryableFields::TxAccessList)
    //    .unwrap();

    let selected_offsets = query_builder.get_selected_offsets();
    let raw = encoded.abi.clone();

    let expected_results: Vec<ResultField> = vec![
        ResultField::TxMaxPriorityFeePerGas(tx.max_priority_fee_per_gas().unwrap_or(0)),
        ResultField::TxMaxFeePerGas(tx.max_fee_per_gas()),
        // TODO: Properly test access list
        //ResultField::TxAccessList(tx.access_list().unwrap()),
    ];

    // Checking that all result data matches expected
    check_results(expected_results, selected_offsets, raw.clone());
}

// There are no new fields in type 3 transactions for which we want to support bridging.
// We instead make sure the query builder works for some type 2 transaction fields.
// - TxMaxPriorityFeePerGas
// - TxMaxFeePerGas
#[tokio::test]
async fn type_3_tx_queried_fields_match_expected() {
    // Get legacy transaction via rpc
    let (tx, rx) = get_transaction_and_receipt(
        "0x085d2fe01372711005b053a1b0d081c13cde19b6ddb77cae847e0d11a0a0cafe",
    )
    .await;
    assert!(tx.inner.is_eip4844());

    // Encode transaction
    let encoded = abi_encode(tx.clone(), rx.clone()).unwrap();

    let mut query_builder = QueryBuilder::create_from_transaction(tx.clone(), rx.clone())
        .expect("creating queryable builder should work");
    query_builder.set_abi_provider(Arc::new(|contract_address| {
        Box::pin(test_abi_provider(contract_address.clone()))
    }));

    query_builder
        .add_static_field(QueryableFields::TxMaxPriorityFeePerGas)
        .unwrap();
    query_builder
        .add_static_field(QueryableFields::TxMaxFeePerGas)
        .unwrap();

    let selected_offsets = query_builder.get_selected_offsets();
    let raw = encoded.abi.clone();

    let expected_results: Vec<ResultField> = vec![
        ResultField::TxMaxPriorityFeePerGas(tx.max_priority_fee_per_gas().unwrap_or(0)),
        ResultField::TxMaxFeePerGas(tx.max_fee_per_gas()),
    ];

    // Checking that all result data matches expected
    check_results(expected_results, selected_offsets, raw.clone());
}

#[tokio::test]
async fn type_4_tx_queried_fields_match_expected() {
    // Get legacy transaction via rpc
    let (tx, rx) = get_transaction_and_receipt(
        "0x2dce846c932bcf50a9dd180e61a09818009da734f2e6761cf9e825f19077f05a",
    )
    .await;
    assert!(tx.inner.is_eip7702());

    // Encode transaction
    let encoded = abi_encode(tx.clone(), rx.clone()).unwrap();

    let mut query_builder = QueryBuilder::create_from_transaction(tx.clone(), rx.clone())
        .expect("creating queryable builder should work");
    query_builder.set_abi_provider(Arc::new(|contract_address| {
        Box::pin(test_abi_provider(contract_address.clone()))
    }));

    //query_builder
    //    .add_static_field(QueryableFields::TxAuthorizationList)
    //    .unwrap();

    let selected_offsets = query_builder.get_selected_offsets();
    let raw = encoded.abi.clone();

    let expected_results: Vec<ResultField> = vec![
        // TODO: Properly test access list
        //ResultField::TxAccessList(tx.authorization_list().unwrap()),
    ];

    // Checking that all result data matches expected
    check_results(expected_results, selected_offsets, raw.clone());
}

async fn test_abi_provider(_contract_address: String) -> Option<String> {

    // hard coded G-CRE's ABI
    let json_str = r#"[{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales730Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"VestingStartDate","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"name","outputs":[{"name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"spender","type":"address"},{"name":"value","type":"uint256"}],"name":"approve","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"totalSupply","outputs":[{"name":"amount","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf365Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"value","type":"uint256"},{"name":"sighash","type":"string"}],"name":"exchange","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"from","type":"address"},{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transferFrom","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf183Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"decimals","outputs":[{"name":"","type":"uint8"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales1095Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf365Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"value","type":"uint256"}],"name":"burn","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf2190Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales183Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale365Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf730Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[],"name":"finalizeSales","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"from","type":"address"},{"name":"value","type":"uint256"}],"name":"burnFrom","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf2190Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf730Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"symbol","outputs":[{"name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale183Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transfer","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"creditcoinSalesLimit","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf1095Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"creditcoinLimitInFrac","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale2190Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale730Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf1095Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"name":"allowance","outputs":[{"name":"remaining","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[],"name":"startVesting","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales2190Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf183Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"IsSalesFinalized","outputs":[{"name":"","type":"bool"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales365Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale1095Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"inputs":[{"name":"creditcoinFoundation","type":"address"},{"name":"devCost","type":"address"}],"payable":false,"stateMutability":"nonpayable","type":"constructor"},{"payable":true,"stateMutability":"payable","type":"fallback"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":false,"name":"value","type":"uint256"},{"indexed":true,"name":"sighash","type":"string"}],"name":"Exchange","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Burnt","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":true,"name":"to","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Transfer","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"owner","type":"address"},{"indexed":true,"name":"spender","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Approval","type":"event"}]"#;

    Some(json_str.into())
}