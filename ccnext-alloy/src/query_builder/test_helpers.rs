use alloy::rpc::types::eth::transaction::Transaction;
use alloy::{
    primitives::{Address, B256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionReceipt,
};
use std::str::FromStr;

pub enum ResultField {
    TransactionStatus(bool),
    EthAddress(Address),
    EventTopic([u8; 32]),
    EventDataField([u8; 32]),
    FunctionSignifier([u8; 4]),
    CallDataField([u8; 32]),
}

pub async fn get_transaction_and_receipt(tx_hash_str: &str) -> (Transaction, TransactionReceipt) {
    // RPC.
    let rpc_url = "https://sepolia-proxy-rpc.creditcoin.network";
    let provider = ProviderBuilder::new().on_http(rpc_url.parse().unwrap());

    // which transaction.
    let tx_hash = B256::from_str(tx_hash_str).unwrap();

    // get the transaction & receipt.
    let tx = provider
        .get_transaction_by_hash(tx_hash)
        .await
        .unwrap()
        .unwrap();
    let rx = provider
        .get_transaction_receipt(tx_hash)
        .await
        .unwrap()
        .unwrap();
    (tx, rx)
}

pub fn check_results(
    expected_results: Vec<ResultField>,
    result_segments: Vec<(usize, usize)>,
    abi: Vec<u8>,
) {
    assert_eq!(expected_results.len(), result_segments.len(), "Number of expected results doesn't match segment count. Expected results: {}, Result segments: {}", expected_results.len(), result_segments.len());

    for (field_number, (expected, (offset, size))) in
        expected_results.iter().zip(result_segments).enumerate()
    {
        // Pad expected result according to type
        let expected_padded: Vec<u8> = match expected {
            ResultField::TransactionStatus(status) => {
                let mut status_padded: Vec<u8> = vec![0; 31];
                status_padded.push(*status as u8);
                status_padded
            }
            ResultField::EthAddress(address) => {
                let mut address_padded: Vec<u8> = vec![0; 12];
                address_padded.append(&mut Vec::from(address.0 .0));
                address_padded
            }
            ResultField::EventTopic(topic) => Vec::from(topic),
            ResultField::EventDataField(field) => Vec::from(field),
            ResultField::FunctionSignifier(signifier) => Vec::from(signifier),
            ResultField::CallDataField(field) => Vec::from(field),
        };
        // Get segment bytes from abi
        let segment_bytes = &abi[offset..offset + size];
        // Compare, printing which field number and type on error.
        // TODO: Possibly allow field names to be added to ResultFields, just to make errors more intelligible.
        assert_eq!(
            &expected_padded, segment_bytes,
            "Expected and actual didn't match. Field num: {}, Expected: {:?}, Actual: {:?}",
            field_number, expected_padded, segment_bytes
        );
    }
}
