use crate::abi::models::QueryableFields;

use alloy::{consensus::TxType, dyn_abi::DynSolType};
use ccnext_abi_encoding::common::EncodingVersion;

mod v1;

pub fn get_all_fields_for_transaction(
    tx_type: TxType,
    encoding: EncodingVersion,
) -> Vec<(QueryableFields, DynSolType)> {
    let (tx_fields, rx_fields) = match encoding {
        EncodingVersion::V1 => (
            v1::get_mapped_field_for_type(tx_type),
            v1::get_mapped_receipt_fields(),
        ),
    };

    tx_fields.into_iter().chain(rx_fields).collect()
}
