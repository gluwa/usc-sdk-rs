use crate::abi::models::QueryableFields;

use alloy::{consensus::TxType, dyn_abi::DynSolType};
use ccnext_abi_encoding::abi::EncodingVersion;

mod v1;

fn get_mapped_receipt_fields() -> Vec<(QueryableFields, DynSolType)> {
    vec![
        (QueryableFields::RxStatus, DynSolType::Uint(8)),
        (QueryableFields::RxGasUsed, DynSolType::Uint(64)),
        (
            QueryableFields::RxLogs,
            DynSolType::Array(
                DynSolType::Tuple(vec![
                    DynSolType::Address,
                    DynSolType::Array(DynSolType::FixedBytes(32).into()),
                    DynSolType::Bytes,
                ])
                .into(),
            ),
        ),
        (QueryableFields::RxLogBlooms, DynSolType::Bytes),
    ]
}

pub fn get_all_fields_for_transaction(
    tx_type: TxType,
    encoding: EncodingVersion,
) -> Vec<(QueryableFields, DynSolType)> {
    let tx_fields = match encoding {
        EncodingVersion::V1 => v1::get_mapped_field_for_type(tx_type),
    };
    let rx_fields = get_mapped_receipt_fields();

    tx_fields.into_iter().chain(rx_fields).collect()
}
