use crate::abi::models::QueryableFields;

use alloy::{consensus::TxType, dyn_abi::DynSolType};

fn get_mapped_fields_for_type0() -> Vec<(QueryableFields, DynSolType)> {
    vec![
        (QueryableFields::Type, DynSolType::Uint(8)),
        (QueryableFields::TxNonce, DynSolType::Uint(64)),
        (QueryableFields::TxGasPrice, DynSolType::Uint(128)),
        (QueryableFields::TxGasLimit, DynSolType::Uint(64)),
        (QueryableFields::TxFrom, DynSolType::Address),
        (QueryableFields::TxTo, DynSolType::Address),
        (QueryableFields::TxValue, DynSolType::Uint(256)),
        (QueryableFields::TxData, DynSolType::Bytes),
        (QueryableFields::TxV, DynSolType::Uint(256)),
        (QueryableFields::TxR, DynSolType::FixedBytes(32)),
        (QueryableFields::TxS, DynSolType::FixedBytes(32)),
    ]
}

fn get_mapped_fields_for_type1() -> Vec<(QueryableFields, DynSolType)> {
    vec![
        (QueryableFields::Type, DynSolType::Uint(8)),
        (QueryableFields::TxChainId, DynSolType::Uint(64)),
        (QueryableFields::TxNonce, DynSolType::Uint(64)),
        (QueryableFields::TxGasPrice, DynSolType::Uint(128)),
        (QueryableFields::TxGasLimit, DynSolType::Uint(64)),
        (QueryableFields::TxFrom, DynSolType::Address),
        (QueryableFields::TxTo, DynSolType::Address),
        (QueryableFields::TxValue, DynSolType::Uint(256)),
        (QueryableFields::TxData, DynSolType::Bytes),
        (
            QueryableFields::TxAccessList,
            DynSolType::Array(
                DynSolType::Tuple(vec![
                    DynSolType::Address,
                    DynSolType::Array(DynSolType::FixedBytes(32).into()),
                ])
                .into(),
            ),
        ),
        (QueryableFields::TxYParity, DynSolType::Uint(8)),
        (QueryableFields::TxR, DynSolType::FixedBytes(32)),
        (QueryableFields::TxS, DynSolType::FixedBytes(32)),
    ]
}

fn get_mapped_fields_for_type2() -> Vec<(QueryableFields, DynSolType)> {
    vec![
        (QueryableFields::Type, DynSolType::Uint(8)),
        (QueryableFields::TxChainId, DynSolType::Uint(64)),
        (QueryableFields::TxNonce, DynSolType::Uint(64)),
        (
            QueryableFields::TxMaxPriorityFeePerGas,
            DynSolType::Uint(128),
        ),
        (QueryableFields::TxMaxFeePerGas, DynSolType::Uint(128)),
        (QueryableFields::TxGasLimit, DynSolType::Uint(64)),
        (QueryableFields::TxFrom, DynSolType::Address),
        (QueryableFields::TxTo, DynSolType::Address),
        (QueryableFields::TxValue, DynSolType::Uint(256)),
        (QueryableFields::TxData, DynSolType::Bytes),
        (
            QueryableFields::TxAccessList,
            DynSolType::Array(
                DynSolType::Tuple(vec![
                    DynSolType::Address,
                    DynSolType::Array(DynSolType::FixedBytes(32).into()),
                ])
                .into(),
            ),
        ),
        (QueryableFields::TxYParity, DynSolType::Uint(8)),
        (QueryableFields::TxR, DynSolType::FixedBytes(32)),
        (QueryableFields::TxS, DynSolType::FixedBytes(32)),
    ]
}

fn get_mapped_fields_for_type3() -> Vec<(QueryableFields, DynSolType)> {
    vec![
        (QueryableFields::Type, DynSolType::Uint(8)),
        (QueryableFields::TxChainId, DynSolType::Uint(64)),
        (QueryableFields::TxNonce, DynSolType::Uint(64)),
        (
            QueryableFields::TxMaxPriorityFeePerGas,
            DynSolType::Uint(128),
        ),
        (QueryableFields::TxMaxFeePerGas, DynSolType::Uint(128)),
        (QueryableFields::TxGasLimit, DynSolType::Uint(64)),
        (QueryableFields::TxFrom, DynSolType::Address),
        (QueryableFields::TxTo, DynSolType::Address),
        (QueryableFields::TxValue, DynSolType::Uint(256)),
        (QueryableFields::TxData, DynSolType::Bytes),
        (
            QueryableFields::TxAccessList,
            DynSolType::Array(
                DynSolType::Tuple(vec![
                    DynSolType::Address,
                    DynSolType::Array(DynSolType::FixedBytes(32).into()),
                ])
                .into(),
            ),
        ),
        (QueryableFields::TxMaxFeePerBlobGas, DynSolType::Uint(128)),
        (
            QueryableFields::TxBlobVersionedHashes,
            DynSolType::Array(DynSolType::FixedBytes(32).into()),
        ),
        (QueryableFields::TxYParity, DynSolType::Uint(8)),
        (QueryableFields::TxR, DynSolType::FixedBytes(32)),
        (QueryableFields::TxS, DynSolType::FixedBytes(32)),
    ]
}

fn get_mapped_fields_for_type4() -> Vec<(QueryableFields, DynSolType)> {
    vec![
        (QueryableFields::Type, DynSolType::Uint(8)),
        (QueryableFields::TxChainId, DynSolType::Uint(64)),
        (QueryableFields::TxNonce, DynSolType::Uint(64)),
        (
            QueryableFields::TxMaxPriorityFeePerGas,
            DynSolType::Uint(128),
        ),
        (QueryableFields::TxMaxFeePerGas, DynSolType::Uint(128)),
        (QueryableFields::TxGasLimit, DynSolType::Uint(64)),
        (QueryableFields::TxFrom, DynSolType::Address),
        (QueryableFields::TxTo, DynSolType::Address),
        (QueryableFields::TxValue, DynSolType::Uint(256)),
        (QueryableFields::TxData, DynSolType::Bytes),
        (
            QueryableFields::TxAccessList,
            DynSolType::Array(
                DynSolType::Tuple(vec![
                    DynSolType::Address,
                    DynSolType::Array(DynSolType::FixedBytes(32).into()),
                ])
                .into(),
            ),
        ),
        (
            QueryableFields::TxSignedAuthorizations,
            // compare to encode_authorization_list
            DynSolType::Array(
                DynSolType::Tuple(vec![
                    DynSolType::Uint(256),
                    DynSolType::Address,
                    DynSolType::Uint(64),
                    DynSolType::Uint(8),
                    DynSolType::Uint(256),
                    DynSolType::Uint(256),
                ])
                .into(),
            ),
        ),
        (QueryableFields::TxYParity, DynSolType::Uint(8)),
        (QueryableFields::TxR, DynSolType::FixedBytes(32)),
        (QueryableFields::TxS, DynSolType::FixedBytes(32)),
    ]
}

pub(super) fn get_mapped_field_for_type(
    transaction_type: TxType,
) -> Vec<(QueryableFields, DynSolType)> {
    match transaction_type {
        TxType::Legacy => get_mapped_fields_for_type0(),
        TxType::Eip2930 => get_mapped_fields_for_type1(),
        TxType::Eip1559 => get_mapped_fields_for_type2(),
        TxType::Eip4844 => get_mapped_fields_for_type3(),
        TxType::Eip7702 => get_mapped_fields_for_type4(),
    }
}
