use crate::{
    encoding::abi::abi_encode,
    my_abi_provider,
    query_builder::abi::{models::QueryableFields, query_builder::QueryBuilder},
    query_builder::test_helpers::{check_results, get_transaction_and_receipt, ResultField},
};
use alloy::consensus::Transaction;
use std::sync::Arc;

// Tx/Rx Fields queried in this test:
// - Rx Status
// - Tx From
// - Tx To (contract addr)
// - Transfer event:
//     - Event addr (contract addr)
//     - Event index 0 (signature)
//     - Event index 1 (from address)
//     - Event index 2 (to, burn address)
//     - Event data field 0 (burned amount)
// - Call Data:
//     - Function signature
//     - data field 0 (burned amount)
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
        Box::pin(my_abi_provider(contract_address.clone()))
    }));

    query_builder
        .add_static_field(QueryableFields::RxStatus)
        .expect("Should work to add from field");
    query_builder
        .add_static_field(QueryableFields::TxFrom)
        .expect("Should work to add from field");
    query_builder
        .add_static_field(QueryableFields::TxTo)
        .expect("Should work to add from field");

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
        ResultField::TransactionStatus(rx.status()),
        ResultField::EthAddress(tx.from), // Caller address
        ResultField::EthAddress(tx.to().expect("Should be to field in contract call")), // Contract address in call
        ResultField::EthAddress(rx.inner.logs()[1].address()), // Contract address in event
        ResultField::EventTopic(rx.inner.logs()[1].topic0().unwrap().0), // Event signature
        ResultField::EventTopic(rx.inner.logs()[1].topics()[1].0), // Event from address
        ResultField::EventTopic(rx.inner.logs()[1].topics()[2].0), // Event to address
        ResultField::EventDataField(
            rx.inner.logs()[1].data().data[..]
                .try_into()
                .expect("Data should contain 1 32 byte field for this transaction"),
        ), // Event value
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
