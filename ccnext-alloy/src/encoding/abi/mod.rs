
use super::common::{
    compute_v, compute_y_parity, encode_blob_hashes,
    AbiEncodeResult,
};
use alloy::{
    consensus::{
        Signed, Transaction as ConsensusTransaction, TxEip1559, TxEip2930,
        TxEip4844Variant, TxEip7702, TxEnvelope, TxLegacy,
    }, dyn_abi::DynSolValue, eips::eip7702::SignedAuthorization, primitives::{Address, B256, U256}, rpc::types::{AccessListItem, Transaction, TransactionReceipt}
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("Custom error: {0}")]
    Custom(String),
}

pub fn encode_transaction_type_0(tx: Transaction, signed_tx: Signed<TxLegacy>) -> Vec<DynSolValue> {

    // Extract transaction fields
    let signature = signed_tx.signature();
    let chain_id = tx.chain_id();
    let v = compute_v(signature, chain_id);
    
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
        TxEnvelope::Eip7702(signed_tx) => encode_transaction_type_4(tx, signed_tx)
    }
}

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

pub fn encode_authorization_list(signed_authorizations: Vec<SignedAuthorization>) -> DynSolValue {
    let mut result = Vec::new();
    for signed_authorization in signed_authorizations {

        let signed_authorization_tuple = DynSolValue::Tuple(vec![
            DynSolValue::Uint(U256::from(signed_authorization.chain_id().clone()), 256),
            DynSolValue::Address(signed_authorization.address().clone()),
            DynSolValue::Uint(U256::from(signed_authorization.nonce()), 64),
            DynSolValue::Uint(U256::from(signed_authorization.y_parity()), 8),
            DynSolValue::Uint(U256::from(signed_authorization.r()), 256),
            DynSolValue::Uint(U256::from(signed_authorization.s()), 256)
        ]);
        
        //let pack_encoded = signed_authorization_tuple.abi_encode_packed();
        result.push(signed_authorization_tuple);
    }

    DynSolValue::Array(result)
}

pub fn encode_access_list(access_list: Vec<AccessListItem>) -> DynSolValue {

    let mut list = Vec::new();
    for access_list_item in access_list {

        let mut storage_keys = Vec::new();
        for storage_item in access_list_item.storage_keys {
            storage_keys.push(DynSolValue::FixedBytes(storage_item, 32));
        }

        // Create the `DynSolValue::Tuple` (address, storage_keys)
        let access_list_tuple = DynSolValue::Tuple(vec![
            DynSolValue::Address(access_list_item.address),  // Address
            DynSolValue::Array(storage_keys)          
        ]);

        list.push(access_list_tuple);
    }

    // Wrap into `DynSolValue::Array`
    DynSolValue::Array(list)
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
