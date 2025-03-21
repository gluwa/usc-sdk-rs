use alloy::{consensus::TxType, dyn_abi::DynSolType};
use super::models::QueryableFields;

fn get_mapped_fields_for_type0() -> Vec<(QueryableFields, DynSolType)> {
    return vec![
        ( QueryableFields::Type, DynSolType::Uint(8)),
        ( QueryableFields::TxNonce, DynSolType::Uint(64)),
        ( QueryableFields::TxGasPrice, DynSolType::Uint(128)),
        ( QueryableFields::TxGasLimit, DynSolType::Uint(64)),
        ( QueryableFields::TxFrom, DynSolType::Address),
        ( QueryableFields::TxTo, DynSolType::Address),
        ( QueryableFields::TxValue, DynSolType::Uint(256)),
        ( QueryableFields::TxData, DynSolType::Bytes),
        ( QueryableFields::TxV, DynSolType::Uint(256)),
        ( QueryableFields::TxV, DynSolType::FixedBytes(32)),
        ( QueryableFields::TxV, DynSolType::FixedBytes(32))
    ];
}

fn get_mapped_fields_for_type1() -> Vec<(QueryableFields, DynSolType)> {
    return vec![
        ( QueryableFields::Type, DynSolType::Uint(8)),
        ( QueryableFields::TxChainId, DynSolType::Uint(64)),
        ( QueryableFields::TxNonce, DynSolType::Uint(64)),
        ( QueryableFields::TxGasPrice, DynSolType::Uint(128)),
        ( QueryableFields::TxGasLimit, DynSolType::Uint(64)),
        ( QueryableFields::TxFrom, DynSolType::Address),
        ( QueryableFields::TxTo, DynSolType::Address),
        ( QueryableFields::TxValue, DynSolType::Bytes),
        ( 
            QueryableFields::TxAccessList, 
            DynSolType::Array(
                DynSolType::Tuple(vec![
                    DynSolType::Address, 
                    DynSolType::Array(DynSolType::FixedBytes(32).into())
                ]).into()
            )
        ),
        ( QueryableFields::TxYParity, DynSolType::Uint(256)),
        ( QueryableFields::TxV, DynSolType::FixedBytes(32)),
        ( QueryableFields::TxV, DynSolType::FixedBytes(32))
    ];
}

fn get_mapped_fields_for_type2() -> Vec<(QueryableFields, DynSolType)> {
    return vec![
        ( QueryableFields::Type, DynSolType::Uint(8)),
        ( QueryableFields::TxChainId, DynSolType::Uint(64)),
        ( QueryableFields::TxNonce, DynSolType::Uint(64)),
        ( QueryableFields::TxMaxPriorityFeePerGas, DynSolType::Uint(128)),
        ( QueryableFields::TxMaxFeePerGas, DynSolType::Uint(128)),
        ( QueryableFields::TxGasLimit, DynSolType::Uint(64)),
        ( QueryableFields::TxFrom, DynSolType::Address),
        ( QueryableFields::TxTo, DynSolType::Address),
        ( QueryableFields::TxValue, DynSolType::Uint(256)),
        ( QueryableFields::TxData, DynSolType::Bytes),
        ( 
            QueryableFields::TxAccessList, 
            DynSolType::Array(
                DynSolType::Tuple(vec![
                    DynSolType::Address, 
                    DynSolType::Array(DynSolType::FixedBytes(32).into())
                ]).into()
            )
        ),
        ( QueryableFields::TxYParity, DynSolType::Uint(256)),
        ( QueryableFields::TxV, DynSolType::FixedBytes(32)),
        ( QueryableFields::TxV, DynSolType::FixedBytes(32))
    ];
}

fn get_mapped_fields_for_type3() -> Vec<(QueryableFields, DynSolType)> 
{
    return vec![
        ( QueryableFields::Type, DynSolType::Uint(8)),
        ( QueryableFields::TxChainId, DynSolType::Uint(64)),
        ( QueryableFields::TxNonce, DynSolType::Uint(64)),
        ( QueryableFields::TxMaxPriorityFeePerGas, DynSolType::Uint(128)),
        ( QueryableFields::TxMaxFeePerGas, DynSolType::Uint(128)),
        ( QueryableFields::TxGasLimit, DynSolType::Uint(64)),
        ( QueryableFields::TxFrom, DynSolType::Address),
        ( QueryableFields::TxTo, DynSolType::Address),
        ( QueryableFields::TxValue, DynSolType::Uint(256)),
        ( QueryableFields::TxData, DynSolType::Bytes),
        ( 
            QueryableFields::TxAccessList, 
            DynSolType::Array(
                DynSolType::Tuple(vec![
                    DynSolType::Address, 
                    DynSolType::Array(DynSolType::FixedBytes(32).into())
                ]).into()
            )
        ),
        ( QueryableFields::TxYParity, DynSolType::Uint(256)),
        ( QueryableFields::TxV, DynSolType::FixedBytes(32)),
        ( QueryableFields::TxV, DynSolType::FixedBytes(32))
    ];
}

fn get_mapped_fields_for_type4() -> Vec<(QueryableFields, DynSolType)>  
{
    return vec![
        ( QueryableFields::Type, DynSolType::Uint(8)),
        ( QueryableFields::TxChainId, DynSolType::Uint(64)),
        ( QueryableFields::TxNonce, DynSolType::Uint(64)),
        ( QueryableFields::TxMaxPriorityFeePerGas, DynSolType::Uint(128)),
        ( QueryableFields::TxMaxFeePerGas, DynSolType::Uint(128)),
        ( QueryableFields::TxGasLimit, DynSolType::Uint(64)),
        ( QueryableFields::TxFrom, DynSolType::Address),
        ( QueryableFields::TxTo, DynSolType::Address),
        ( QueryableFields::TxValue, DynSolType::Bytes),
        ( 
            QueryableFields::TxAccessList, 
            DynSolType::Array(
                DynSolType::Tuple(vec![
                    DynSolType::Address, 
                    DynSolType::Array(DynSolType::FixedBytes(32).into())
                ]).into()
            )
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
                ]).into()
            ),
        ),
        ( QueryableFields::TxYParity, DynSolType::Uint(256)),
        ( QueryableFields::TxV, DynSolType::FixedBytes(32)),
        ( QueryableFields::TxV, DynSolType::FixedBytes(32))
    ];
}

fn get_mapped_field_for_type(transaction_type: TxType) -> Vec<(QueryableFields, DynSolType)> {
    match transaction_type {
        TxType::Legacy => get_mapped_fields_for_type0(),
        TxType::Eip2930 => get_mapped_fields_for_type1(),
        TxType::Eip1559 => get_mapped_fields_for_type2(),
        TxType::Eip4844 => get_mapped_fields_for_type3(),
        TxType::Eip7702 => get_mapped_fields_for_type4(),
    }
}

fn get_mapped_receipt_fields() -> Vec<(QueryableFields, DynSolType)> {
    return vec![
        ( QueryableFields::RxStatus, DynSolType::Uint(8) ),
        ( QueryableFields::RxGasUsed, DynSolType::Uint(64) ),
        ( 
            QueryableFields::RxLogs, 
            DynSolType::Array(
                DynSolType::Tuple(vec![
                    DynSolType::Address,
                    DynSolType::Array(DynSolType::FixedBytes(32).into()),
                    DynSolType::Bytes
                ]).into()
            )    
        ),
        ( QueryableFields::RxLogBlooms, DynSolType::Bytes ),
    ];
}

pub fn get_all_fields_for_transaction(tx_type: TxType) -> Vec<(QueryableFields, DynSolType)>
{
    let tx_fields = get_mapped_field_for_type(tx_type);
    let rx_fields = get_mapped_receipt_fields();
    let mut all_fields = Vec::new();
    all_fields.extend(tx_fields);
    all_fields.extend(rx_fields);
    return all_fields;
}