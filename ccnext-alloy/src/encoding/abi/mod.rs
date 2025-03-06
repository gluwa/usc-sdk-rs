use alloy::{dyn_abi::DynSolType, rpc::types::{Transaction, TransactionReceipt}};
use super::common::{AbiEncodeResult, EncodedFields};
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
            "uint256", "uint256", "uint256", "address", "address", "uint256", "bytes", "uint256", "bytes32", "bytes32"
        ],
        values: [
            tx.nonce, tx.gasPrice, tx.gasLimit, tx.from, addressOrZero(tx.to), tx.value, tx.data, tx.signature.v, tx.signature.r, tx.signature.s
        ]
    };
}
*/
pub fn encode_transaction_type_0(tx: Transaction) -> Result<EncodedFields, Box<dyn std::error::Error>>
{
    let mut types = Vec::new();
    let mut values = Vec::new();



    Ok(EncodedFields {
        types: types,
        values: values
    })
}

pub fn encode_transaction(tx: Transaction) -> Result<EncodedFields, Box<dyn std::error::Error>>
{
    match tx.inner.tx_type() {
        alloy::consensus::TxType::Legacy => {
            encode_transaction_type_0(tx)
        },
        alloy::consensus::TxType::Eip2930 => todo!(),
        alloy::consensus::TxType::Eip1559 => todo!(),
        alloy::consensus::TxType::Eip4844 => todo!(),
        alloy::consensus::TxType::Eip7702 => {
            Err(Box::new(EncodeError::Custom("Eip7002 is not supported".into())))
        }
    }
}

pub fn abi_encode(tx: Transaction, rx: TransactionReceipt) -> Result<AbiEncodeResult, Box<dyn std::error::Error>> 
{
    let _transaction_fields = encode_transaction(tx)?;


    // const txFields = getFieldsForType(tx);
    // const receiptFields = getReceiptFields(rx);
    // const allFieldTypes = [...txFields.types, ...receiptFields.types];
    // const allFieldValues = [...txFields.values, ...receiptFields.values];
    // const abi = AbiCoder.defaultAbiCoder().encode(allFieldTypes, allFieldValues);
    // return {
    //   types: allFieldTypes,
    //   abi
    // }
    Ok(AbiEncodeResult {
       types: vec![DynSolType::Int(8)],
       abi: vec![0x00] 
    })
}