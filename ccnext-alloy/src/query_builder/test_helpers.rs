use crate::encoding::common::{compute_v, compute_y_parity};
use alloy::consensus::Transaction as _;
use alloy::rpc::types::eth::transaction::Transaction;
use alloy::eips::eip2930::AccessList;
use alloy::{
    primitives::{Address, B256, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionReceipt,
};
use std::str::FromStr;

pub enum ResultField {
    TxType(u8),
    TxChainId(u64),
    TxNonce(u64),
    TxGasPrice(u128),
    TxMaxPriorityFeePerGas(u128),
    TxMaxFeePerGas(u128),
    TxGasLimit(u64),
    EthAddress(Address),
    TxValue(U256),
    TxAccessList(AccessList),
    TxV(U256),
    TxYParity(u8),
    TxR([u8; 32]),
    TxS([u8; 32]),
    RxStatus(u8),
    RxGasUsed(u64),
    RxLogBlooms(Vec<u8>),
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
            // All cases where 1 byte is padded to 32 bytes
            ResultField::TxType(value) |
            ResultField::TxYParity(value) | 
            ResultField::RxStatus(value) => {
                let mut value_padded: Vec<u8> = vec![0; 31];
                value_padded.push(*value);
                value_padded
            },
            // All cases where 8 bytes are padded to 32 bytes
            ResultField::RxGasUsed(value) |
            ResultField::TxNonce(value) |
            ResultField::TxChainId(value) |
            ResultField::TxGasLimit(value) => {
                let mut value_padded: Vec<u8> = vec![0; 24];
                value_padded.append(&mut Vec::from(value.to_be_bytes()));
                value_padded
            }
            // All cases where 16 bytes are padded to 32 bytes
            ResultField::TxGasPrice(value) |
            ResultField::TxMaxPriorityFeePerGas(value) | 
            ResultField::TxMaxFeePerGas(value) => {
                let mut value_padded: Vec<u8> = vec![0; 16];
                value_padded.append(&mut Vec::from(value.to_be_bytes()));
                value_padded
            }
            ResultField::EthAddress(address) => {
                let mut address_padded: Vec<u8> = vec![0; 12];
                address_padded.append(&mut Vec::from(address.0 .0));
                address_padded
            }
            ResultField::TxValue(value) => value.to_be_bytes_vec(),
            ResultField::TxAccessList(list) => {
                //TODO: figure out how access list should look when encoded
                vec![] 
            }
            ResultField::TxV(v) => v.to_be_bytes_vec(),
            ResultField::TxR(r) => Vec::from(r),
            ResultField::TxS(s) => Vec::from(s),
            ResultField::RxLogBlooms(blooms) => blooms.clone(),
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

pub fn get_vrs(tx: &Transaction) -> (U256, [u8; 32], [u8; 32]) {
    let signed_tx = tx
        .inner
        .as_legacy()
        .expect("Already checked that tx is legacy");
    let signature = signed_tx.signature();
    let chain_id = tx.chain_id();
    let v = compute_v(signature, chain_id);
    let r: [u8; 32] = signature.r().to_be_bytes::<32>()[0..32].try_into().unwrap();
    let s: [u8; 32] = signature.s().to_be_bytes::<32>()[0..32].try_into().unwrap();
    (v, r, s)
}

pub fn get_y_parity(tx: &Transaction) -> u8 {
    let signature = tx
        .inner
        .signature();
    compute_y_parity(signature)
}
