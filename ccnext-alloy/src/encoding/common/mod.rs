use alloy::dyn_abi::{DynSolType, DynSolValue};
use alloy::dyn_abi::abi::Token;
use alloy::eips::eip7702::SignedAuthorization;
use alloy::primitives::{FixedBytes, B256, U256};
use alloy::rpc::types::{AccessListItem};
use alloy::signers::Signature;


pub struct AbiEncodeResult
{
    pub types: Vec<DynSolType>,
    pub abi: Vec<u8>
}

pub fn compute_v(signature: &Signature, chain_id: Option<u64>) -> U256 {
    let parity = signature.v() as u64; // Get y_parity (boolean as 0 or 1)

    match chain_id {
        Some(id) => U256::from(27 + 2 * id + parity), // EIP-155 format
        None => U256::from(27 + parity), // Legacy format
    }
}

pub fn compute_y_parity(signature: &Signature) -> u8 {
    if signature.v() { 1 } else { 0 }
}

pub fn encode_access_list(access_list: Vec<AccessListItem>) -> DynSolValue {

    let mut list = Vec::new();
    for access_list_item in access_list {

        let mut storage_keys = Vec::new();
        for storage_item in access_list_item.storage_keys {
            storage_keys.push(DynSolValue::FixedBytes(storage_item, 32));
        }

        // Create the `DynSolValue::Tuple` (address, storage_keys)
        list.push(DynSolValue::Tuple(vec![
            DynSolValue::Address(access_list_item.address),  // Address
            DynSolValue::Array(storage_keys)          
        ]));
    }

    // Wrap into `DynSolValue::Array`
    DynSolValue::Array(list)
}

pub fn encode_blob_hashes(blob_hashes: Vec<FixedBytes<32>>) -> DynSolValue {
    let mut result = Vec::new();
    for hash in blob_hashes {
        result.push(DynSolValue::FixedBytes(hash, 32));
    }
    DynSolValue::Array(result)
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
        
        result.push(signed_authorization_tuple);
    }

    DynSolValue::Array(result)
}