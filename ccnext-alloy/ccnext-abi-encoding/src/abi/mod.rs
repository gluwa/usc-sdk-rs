use super::common::AbiEncodeResult;
use alloy::{
    dyn_abi::DynSolValue,
    primitives::U256,
    rpc::types::{Transaction, TransactionReceipt},
};
use thiserror::Error;

mod v1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingVersion {
    V1 = 1,
}

fn encode_receipt(rx: TransactionReceipt) -> Vec<DynSolValue> {
    let log_blooms = rx.inner.logs_bloom().0.to_vec();
    let result = vec![
        DynSolValue::Uint(U256::from(rx.status()), 8),
        DynSolValue::Uint(U256::from(rx.gas_used), 64),
        DynSolValue::Array(
            rx.inner
                .logs()
                .iter()
                .map(|log| {
                    let topics = DynSolValue::Array(
                        log.topics()
                            .iter()
                            .map(|topic| DynSolValue::FixedBytes(*topic, 32))
                            .collect(),
                    );

                    DynSolValue::Tuple(vec![
                        DynSolValue::Address(log.address()),
                        topics,
                        DynSolValue::Bytes(log.data().data.to_vec()),
                    ])
                })
                .collect(),
        ),
        DynSolValue::Bytes(log_blooms),
    ];

    result
}

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("Custom error: {0}")]
    Custom(String),
}
pub fn abi_encode(
    tx: Transaction,
    rx: TransactionReceipt,
    version: EncodingVersion,
) -> Result<AbiEncodeResult, Box<dyn std::error::Error>> {
    let transaction_fields = match version {
        EncodingVersion::V1 => v1::encode_transaction(tx),
    };
    let receipt_fields = encode_receipt(rx);
    let mut all_fields = Vec::new();
    all_fields.extend(transaction_fields);
    all_fields.extend(receipt_fields);
    let tuple = DynSolValue::Tuple(all_fields.clone());
    let final_bytes = match tuple.abi_encode_sequence() {
        Some(final_bytes) => final_bytes,
        None => {
            return Err(Box::new(EncodeError::Custom(
                "Failed to encode sequence".into(),
            )));
        }
    };

    let field_types: Vec<String> = all_fields
        .into_iter()
        .map(|field| match field.as_type() {
            Some(sol_type) => sol_type.sol_type_name().into_owned(),
            None => "unknown".into(),
        })
        .collect();

    Ok(AbiEncodeResult {
        types: field_types,
        abi: final_bytes,
    })
}
