use super::common::AbiEncodeResult;

use alloy::rpc::types::{Transaction, TransactionReceipt};
use thiserror::Error;

mod v1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingVersion {
    V1 = 1,
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
    match version {
        EncodingVersion::V1 => v1::abi_encode(tx, rx),
    }
}
