use crate::{
    encoding::abi::abi_encode,
    my_abi_provider, print_decoded_result,
    query_builder::abi::{
        abi_encoding_mapping::get_all_fields_for_transaction,
        models::{FieldMetadata, QueryableFields},
        query_builder::QueryBuilder,
        utils::compute_abi_offsets,
    },
};
use alloy::{
    consensus::Transaction,
    dyn_abi::{DynSolType, DynSolValue},
    eips::BlockId,
    primitives::{Address, B256, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::BlockTransactionsKind,
};
use serde::Serialize;
use std::{any::Any, fmt::format, fs::File, io::Write, str::FromStr, sync::Arc};
// BASIC TEST
// - We use the query builder to get offsets
// - We use offsets to bet ABI bytes segments
// - We decode ABI bytes segments using expected field types
// - We check the values of those fields against expected values from online tools

// - Create functionality to collect inner types during decode as well as offsets.

#[tokio::test]
async fn basic_queried_fields_match_expected() {
    // RPC.
    let rpc_url = "https://sepolia-proxy-rpc.creditcoin.network";
    let provider = ProviderBuilder::new().on_http(rpc_url.parse().unwrap());

    // which transaction.
    let tx_hash_str = "0xc990ce703dd3ca83429c302118f197651678de359c271f205b9083d4aa333aae";
    let tx_hash = B256::from_str(tx_hash_str).unwrap();

    // get the transaction & receipt.
    let tx = provider
        .get_transaction_by_hash(tx_hash)
        .await
        .unwrap()
        .unwrap();
    let rx = provider
        .get_transaction_receipt(tx_hash)
        .await
        .unwrap()
        .unwrap();

    // encode this transaction.
    let encoded = abi_encode(tx.clone(), rx.clone()).unwrap();
    println!("\nEncoded ABI:\n{}", hex::encode(encoded.abi.clone()));

    // get the types of this transaction..
    let field_and_types = get_all_fields_for_transaction(tx.inner.tx_type());
    println!("\nField and types:\n {:?}", field_and_types);

    // the solidity types expected for this transaction :)
    let types_only: Vec<DynSolType> = field_and_types.iter().map(|f| f.1.clone()).collect();
    println!("\nTypes only:\n {:?}", types_only);

    // decode the offsets of the transaction.
    match compute_abi_offsets(types_only, encoded.abi.clone()) {
        Ok(offsets) => {
            println!("\n\nComputed ABI Offsets:");
            print_decoded_result(offsets, "".into());
        }
        Err(decoded_error) => {
            println!("Failed to decode {:?}", decoded_error);
        }
    }

    let mut query_builder = QueryBuilder::create_from_transaction(tx.clone(), rx.clone())
        .expect("creating queryable builder should work");
    query_builder.set_abi_provider(Arc::new(|contract_address| {
        Box::pin(my_abi_provider(contract_address.clone()))
    }));

    query_builder
        .add_static_field(QueryableFields::RxStatus)
        .expect("Should work to add from field");
    query_builder
        .add_static_field(QueryableFields::TxFrom)
        .expect("Should work to add from field");
    query_builder
        .add_static_field(QueryableFields::TxTo)
        .expect("Should work to add from field");

    // just to keep it simple for now
    // i'll just say i care about the event index 1
    // which is the transfer event from, to (zero address), amount
    query_builder
        .event_builder(
            "Transfer".into(),
            |log, event, log_index| log_index == 1,
            |builder| {
                // fun no? :)
                builder
                    .add_address()
                    .unwrap()
                    .add_signature()
                    .unwrap()
                    .add_argument("from")
                    .unwrap()
                    .add_argument("to")
                    .unwrap()
                    .add_argument("value")
                    .unwrap();

                Ok(())
            },
        )
        .await
        .expect("should have matched an event and constructed offsets");

    query_builder
        .function_builder("burn".into(), |b| {
            b.add_signature()
                .unwrap()
                .add_argument("value".into())
                .unwrap();
            Ok(())
        })
        .await
        .expect("should be able to query some calldata segments");

    println!("\nResult of query segments:");
    let selected_offsets = query_builder.get_selected_offsets();
    let raw = encoded.abi.clone();
    for (offset, size) in selected_offsets.clone() {
        let slice = &raw[offset..offset + size];
        println!("({}, {})\t 0x{}", offset, size, hex::encode(slice));
    }

    let expected_status = rx.status() as u8;
    let expected_from = tx.from;
    let expected_to = tx.to().expect("Should be to field in contract call");
    let expected_event_address = rx.inner.logs()[1].address();
    let expected_event_signature = rx.inner.logs()[1].topic0().unwrap().0;
    let expected_event_from = rx.inner.logs()[1].topics()[1].0;
    let expected_event_to = rx.inner.logs()[1].topics()[2].0;
    let expected_event_value = &rx.inner.logs()[1].data().data[..];
    assert!(tx.inner.is_legacy());
    let expected_function_sig: &[u8] = &tx.inner.as_legacy().unwrap().tx().input[0..4];
    let expected_function_value: &[u8] = &tx.inner.as_legacy().unwrap().tx().input[4..36];

    // Checking validity of status segment
    let mut status_padded: Vec<u8> = vec![0; 31];
    status_padded.push(expected_status);
    let (offset, size) = selected_offsets[0];
    let segment_bytes = &raw[offset..offset + size];
    assert_eq!(&status_padded, segment_bytes);
    // Checking validity of tx from segment
    let mut from_padded: Vec<u8> = vec![0; 12];
    from_padded.append(&mut Vec::from(expected_from.0.0));
    let (offset, size) = selected_offsets[1];
    let segment_bytes = &raw[offset..offset + size];
    assert_eq!(&from_padded, segment_bytes);
    // Checking validity of tx to segment
    let mut to_padded: Vec<u8> = vec![0; 12];
    to_padded.append(&mut Vec::from(expected_to.0.0));
    let (offset, size) = selected_offsets[2];
    let segment_bytes = &raw[offset..offset + size];
    assert_eq!(&to_padded, segment_bytes);
    // Checking event address
    let mut evt_addr_padded: Vec<u8> = vec![0; 12];
    evt_addr_padded.append(&mut Vec::from(expected_event_address.0.0));
    let (offset, size) = selected_offsets[3];
    let segment_bytes = &raw[offset..offset + size];
    assert_eq!(&evt_addr_padded, segment_bytes);
    // Checking event signature, doesn't need padding
    let (offset, size) = selected_offsets[4];
    let segment_bytes = &raw[offset..offset + size];
    assert_eq!(&expected_event_signature, segment_bytes);
    // Checking from field of event
    let (offset, size) = selected_offsets[5];
    let segment_bytes = &raw[offset..offset + size];
    assert_eq!(&expected_event_from, segment_bytes);
    // Checking to field of event
    let (offset, size) = selected_offsets[6];
    let segment_bytes = &raw[offset..offset + size];
    assert_eq!(&expected_event_to, segment_bytes);
    // Checking value field of event
    let (offset, size) = selected_offsets[7];
    let segment_bytes = &raw[offset..offset + size];
    assert_eq!(expected_event_value, segment_bytes);
    // Checking function signature
    let (offset, size) = selected_offsets[8];
    let segment_bytes = &raw[offset..offset + size];
    assert_eq!(expected_function_sig, segment_bytes);
    // Checking value parameter to function
    let (offset, size) = selected_offsets[9];
    let segment_bytes = &raw[offset..offset + size];
    assert_eq!(expected_function_value, segment_bytes);
}
