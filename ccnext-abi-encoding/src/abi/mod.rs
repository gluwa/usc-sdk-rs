use super::common::{AbiEncodeResult, EncodingVersion};

use alloy::rpc::types::{Transaction, TransactionReceipt};
use thiserror::Error;

mod v1;

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("Custom error: {0}")]
    Custom(String),
}

/// Encodes a given ethereum transaction and its receipt into ABI format
/// according to the specified encoding version.
///
/// This function assumes that both the transaction and receipt comply with the ethereum specifications
/// as defined in the `alloy` crate.
pub fn abi_encode(
    tx: Transaction,
    rx: TransactionReceipt,
    version: EncodingVersion,
) -> Result<AbiEncodeResult, Box<dyn std::error::Error>> {
    match version {
        EncodingVersion::V1 => v1::abi_encode(tx, rx),
    }
}
