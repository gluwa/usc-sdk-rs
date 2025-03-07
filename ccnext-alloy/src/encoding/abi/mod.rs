use std::any::Any;

use super::common::{
    compute_v, compute_y_parity, encode_access_list, encode_authorization_list, encode_blob_hashes,
    AbiEncodeResult,
};
use alloy::{
    consensus::{
        Signed, Transaction as ConsensusTransaction, TxEip1559, TxEip2930,
        TxEip4844Variant, TxEip7702, TxEnvelope, TxLegacy,
    }, dyn_abi::{DynSolType, DynSolValue}, primitives::{Address, B256, U256}, rpc::types::{Transaction, TransactionReceipt}
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("Custom error: {0}")]
    Custom(String),
}

/*
function getFieldsForType0(tx: TransactionResponse): EncodedFields {
  return {
    types: [
      "uint8", "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "uint256", "bytes32", "bytes32"
    ],
    values: [
      tx.type, tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, tx.signature.v, tx.signature.r, tx.signature.s
    ]
  };
}
*/
pub fn encode_transaction_type_0(tx: Transaction, signed_tx: Signed<TxLegacy>) -> Vec<DynSolValue> {
    // keep this to use later.
    // let types = vec![
    //     DynSolType::Uint(8),
    //     DynSolType::Uint(64),
    //     DynSolType::Uint(128),
    //     DynSolType::Uint(64),
    //     DynSolType::Address,
    //     DynSolType::Address,
    //     DynSolType::Uint(256),
    //     DynSolType::Bytes,
    //     DynSolType::Uint(256),
    //     DynSolType::FixedBytes(32),
    //     DynSolType::FixedBytes(32)
    // ];

    // Extract transaction fields
    let signature = signed_tx.signature();
    let chain_id = tx.chain_id();
    let v = compute_v(signature, chain_id);

    println!("0x{}", hex::encode(tx.input().to_vec()));

    let values: Vec<DynSolValue> = vec![
        DynSolValue::Uint(U256::from(0), 8),                            // Transaction type 0
        DynSolValue::Uint(U256::from(signed_tx.tx().nonce), 64),        // Nonce
        DynSolValue::Uint(U256::from(signed_tx.tx().gas_price), 128),   // Gas price
        DynSolValue::Uint(U256::from(signed_tx.tx().gas_limit), 64),    // Gas limit
        DynSolValue::Address(tx.from),                                  // From address
        DynSolValue::Address(tx.to().unwrap_or(Address::ZERO)),         // To address
        DynSolValue::Uint(tx.value(), 256), // Value                           // Value
        DynSolValue::Bytes(tx.input().to_vec()), // Input data (true = dynamic encoding)
        DynSolValue::Uint(U256::from(v), 256),                          // v (legacy is uint8)
        DynSolValue::FixedBytes(B256::from(signature.r()), 32),         // r
        DynSolValue::FixedBytes(B256::from(signature.s()), 32)          // s
    ];

    values
}

pub fn encode_transaction_type_1(
    tx: Transaction,
    signed_tx: Signed<TxEip2930>,
) -> Vec<DynSolValue> {
    // Extract transaction fields
    let signature = signed_tx.signature();
    let y_parity: u8 = compute_y_parity(signature);
    let access_list = encode_access_list(signed_tx.tx().access_list.0.clone());

    let values: Vec<DynSolValue> = vec![
        DynSolValue::Uint(U256::from(1), 8), // Transaction type 1
        DynSolValue::Uint(U256::from(signed_tx.tx().chain_id), 64),
        DynSolValue::Uint(U256::from(signed_tx.tx().nonce), 64), // Nonce
        DynSolValue::Uint(U256::from(signed_tx.tx().gas_price), 128), // Gas price
        DynSolValue::Uint(U256::from(signed_tx.tx().gas_limit), 64), // Gas limit
        DynSolValue::Address(tx.from),       // From address
        DynSolValue::Address(tx.to().unwrap_or(Address::ZERO)), // To address
        DynSolValue::Uint(tx.value(), 256),  // Value
        DynSolValue::Bytes(tx.input().to_vec()), // Input data (true = dynamic encoding)
        access_list,
        DynSolValue::Uint(U256::from(y_parity), 8),     // y parity
        DynSolValue::FixedBytes(B256::from(signature.r()), 32), // r
        DynSolValue::FixedBytes(B256::from(signature.s()), 32), // s
    ];

    values
}

/*
function getFieldsForType2(tx: TransactionResponse): EncodedFields {
  return {
    types: [
      "uint8", "uint64", "uint256", "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "tuple(address,bytes32[])[]", "uint8", "bytes32", "bytes32"
    ],
    values: [
      tx.type, tx.chainId, tx.nonce, tx.maxPriorityFeePerGas, tx.maxFeePerGas, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, encodeAccessList(tx.accessList), tx.signature.yParity, tx.signature.r, tx.signature.s
    ]
  };
}
 */
pub fn encode_transaction_type_2(
    tx: Transaction,
    signed_tx: Signed<TxEip1559>,
) -> Vec<DynSolValue> {
    // Extract transaction fields
    let signature = signed_tx.signature();
    let y_parity = compute_y_parity(signature);
    let access_list = encode_access_list(signed_tx.tx().access_list.0.clone());

    let values: Vec<DynSolValue> = vec![
        DynSolValue::Uint(U256::from(2), 8),
        DynSolValue::Uint(U256::from(signed_tx.tx().chain_id), 64),
        DynSolValue::Uint(U256::from(signed_tx.tx().nonce), 64),
        DynSolValue::Uint(U256::from(tx.max_priority_fee_per_gas().unwrap_or(0)), 128),
        DynSolValue::Uint(U256::from(tx.max_fee_per_gas()), 128),
        DynSolValue::Uint(U256::from(signed_tx.tx().gas_limit), 64),
        DynSolValue::Address(tx.from),
        DynSolValue::Address(tx.to().unwrap_or(Address::ZERO)),
        DynSolValue::Uint(tx.value(), 256),
        DynSolValue::Bytes(tx.input().to_vec()),
        access_list,
        DynSolValue::Uint(U256::from(y_parity), 8),
        DynSolValue::FixedBytes(B256::from(signature.r()), 32),
        DynSolValue::FixedBytes(B256::from(signature.s()), 32),
    ];

    values
}

/*
function getFieldsForType3(tx: TransactionResponse): EncodedFields {
  const out = {
    types: [
      "uint8", "uint256", "uint256", "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "tuple(address,uint256[])[]", "uint256", "bytes32[]", "uint8", "bytes32", "bytes32"
    ],
    values: [
      tx.type, tx.chainId, tx.nonce, tx.maxPriorityFeePerGas, tx.maxFeePerGas, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, encodeAccessList(tx.accessList), tx.maxFeePerBlobGas, tx.blobVersionedHashes, tx.signature.yParity, tx.signature.r, tx.signature.s
    ]
  };

  return out;
}
   */
pub fn encode_transaction_type_3(
    tx: Transaction,
    signed_tx: Signed<TxEip4844Variant>,
) -> Vec<DynSolValue> {
    // Extract transaction fields
    let signature = signed_tx.signature();
    let y_parity = compute_y_parity(signature);

    let access_list_item_vector = match tx.access_list() {
        Some(access_list) => access_list.0.clone(),
        None => Vec::new(),
    };
    let access_list = encode_access_list(access_list_item_vector);
    let blob_version_hashes = tx.blob_versioned_hashes().unwrap_or_default();
    let blob_hashes = encode_blob_hashes(blob_version_hashes.to_vec());

    // it seems its possible that chain id isn't set for some reason?
    // if its a side car blob transaction.
    let chain_id = tx.chain_id().unwrap_or(0);

    let values: Vec<DynSolValue> = vec![
        DynSolValue::Uint(U256::from(3), 8),
        DynSolValue::Uint(U256::from(chain_id), 64),
        DynSolValue::Uint(U256::from(signed_tx.tx().nonce()), 64),
        DynSolValue::Uint(U256::from(tx.max_priority_fee_per_gas().unwrap_or(0)), 128),
        DynSolValue::Uint(U256::from(tx.max_fee_per_gas()), 128),
        DynSolValue::Uint(U256::from(signed_tx.tx().gas_limit()), 64),
        DynSolValue::Address(tx.from),
        DynSolValue::Address(tx.to().unwrap_or(Address::ZERO)),
        DynSolValue::Uint(tx.value(), 256),
        DynSolValue::Bytes(tx.input().to_vec()),
        access_list,
        DynSolValue::Uint(
            U256::from(signed_tx.tx().max_fee_per_blob_gas().unwrap_or(0u128)),
            128,
        ),
        blob_hashes,
        DynSolValue::Uint(U256::from(y_parity), 8),
        DynSolValue::FixedBytes(B256::from(signature.r()), 32),
        DynSolValue::FixedBytes(B256::from(signature.s()), 32),
    ];

    values
}

pub fn encode_transaction_type_4(
    tx: Transaction,
    signed_tx: Signed<TxEip7702>,
) -> Vec<DynSolValue> {
    // Extract transaction fields
    let signature = signed_tx.signature();
    let y_parity = compute_y_parity(signature);
    let access_list = encode_access_list(signed_tx.tx().access_list.0.clone());
    let authorization_list = encode_authorization_list(signed_tx.tx().authorization_list.clone());

    // it seems its possible that chain id isn't set for some reason?
    // if its a side car blob transaction.
    let chain_id = tx.chain_id().unwrap_or(0);

    let values: Vec<DynSolValue> = vec![
        DynSolValue::Uint(U256::from(3), 8),
        DynSolValue::Uint(U256::from(chain_id), 64),
        DynSolValue::Uint(U256::from(signed_tx.tx().nonce()), 64),
        DynSolValue::Uint(U256::from(tx.max_priority_fee_per_gas().unwrap_or(0)), 128),
        DynSolValue::Uint(U256::from(tx.max_fee_per_gas()), 128),
        DynSolValue::Uint(U256::from(signed_tx.tx().gas_limit()), 64),
        DynSolValue::Address(tx.from),
        DynSolValue::Address(tx.to().unwrap_or(Address::ZERO)),
        DynSolValue::Uint(tx.value(), 256),
        DynSolValue::Bytes(tx.input().to_vec()),
        access_list,
        authorization_list,
        DynSolValue::Uint(U256::from(y_parity), 8),
        DynSolValue::FixedBytes(B256::from(signature.r()), 32),
        DynSolValue::FixedBytes(B256::from(signature.s()), 32),
    ];

    values
}

pub fn encode_transaction(tx: Transaction) -> Vec<DynSolValue> {
    match tx.inner.clone() {
        TxEnvelope::Legacy(signed_tx) => encode_transaction_type_0(tx, signed_tx),
        TxEnvelope::Eip2930(signed_tx) => encode_transaction_type_1(tx, signed_tx),
        TxEnvelope::Eip1559(signed_tx) => encode_transaction_type_2(tx, signed_tx),
        TxEnvelope::Eip4844(signed_tx) => encode_transaction_type_3(tx, signed_tx),
        TxEnvelope::Eip7702(signed_tx) => encode_transaction_type_4(tx, signed_tx),
    }
}

fn encode_dyn_sol_values(values: Vec<DynSolValue>) -> Vec<u8> {
    let mut encoded_bytes = Vec::new();

    for value in values.iter() {
        encoded_bytes.extend(value.abi_encode());
    }

    encoded_bytes
}

/*
function getReceiptFields(rx: TransactionReceipt): EncodedFields {
  return {
    types: [
      "uint256", "uint256", "tuple(address, bytes32[], bytes)[]", "bytes"
    ],
    values: [
      rx.status, rx.gasUsed, rx.logs.map(log => [log.address, log.topics, log.data]), rx.logsBloom
    ]
  };
}
*/
fn encode_receipt(rx: TransactionReceipt) -> Vec<DynSolValue> {

    let log_blooms = rx.inner.logs_bloom().0.to_vec();
    let result = vec![
        DynSolValue::Uint(U256::from(rx.status()), 8),
        DynSolValue::Uint(U256::from(rx.gas_used), 64),
        DynSolValue::Array(
            rx.inner.logs().into_iter().map(|log| {

                let topics = DynSolValue::Array(log.topics().into_iter().map(|topic| {
                    DynSolValue::FixedBytes(topic.clone(), 32)
                }).collect());

                DynSolValue::Tuple(vec![
                    DynSolValue::Address(log.address()),
                    topics,
                    DynSolValue::Bytes(log.data().data.to_vec())
                ])
            }).collect()
        ),
        DynSolValue::Bytes(log_blooms),
    ];

    result
}

pub fn abi_encode(
    tx: Transaction,
    rx: TransactionReceipt,
) -> Result<AbiEncodeResult, Box<dyn std::error::Error>> {

    let transaction_fields = encode_transaction(tx);
    let receipt_fields = encode_receipt(rx);
    let mut all_fields = Vec::new();
    all_fields.extend(transaction_fields);
    all_fields.extend(receipt_fields);
    let tuple = DynSolValue::Tuple(all_fields.clone());
    let final_bytes = match tuple.abi_encode_sequence() {
        Some(final_bytes) => {
            final_bytes
        },
        None => {
            return Err(Box::new(EncodeError::Custom("Failed to encore sequence".into())));
        }
    };

    let field_type: Vec<String> = all_fields.into_iter().map(|field| {

        match field.as_type() {
            Some(sol_type) => {
                sol_type.sol_type_name().into_owned()
            },
            None => "unknown".into()
        }

    }).collect();

    Ok(AbiEncodeResult {
        types: field_type,
        abi: final_bytes,
    })
}
