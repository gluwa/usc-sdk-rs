use std::{any::Any, fmt::format, fs::File, io::Write, str::FromStr, sync::Arc};
use alloy::{dyn_abi::{DynSolType, DynSolValue}, eips::BlockId, primitives::{Address, B256, U256}, providers::{Provider, ProviderBuilder}, rpc::types::{BlockTransactionsKind}};
use encoding::abi::abi_encode;
use query_builder::abi::{abi_encoding_mapping::get_all_fields_for_transaction, models::{FieldMetadata, QueryableFields}, query_builder::QueryBuilder, utils::compute_abi_offsets};

mod encoding;
mod query_builder;

async fn _encode_transaction() -> Result<(), Box<dyn std::error::Error>> {

    let rpc_url = "https://sepolia-proxy-rpc.creditcoin.network";
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    let _type_3 = "0x085d2fe01372711005b053a1b0d081c13cde19b6ddb77cae847e0d11a0a0cafe";
    let _type_2 = "0xdfba59b94bac3da5af5d0fa8b81ae3199069fa6f38002be58c14e94a051e0642";
    let _legacy = "0x0b50111d729c00bac4a99702b2c88e425321c8f8214bc3272072c730d5ff9ad2";
    let _type_4 = "0x2dce846c932bcf50a9dd180e61a09818009da734f2e6761cf9e825f19077f05a";
    let _type_1 = "0x5c8c6d8c61bd8109ce02717db62b12554c097d156b66e30ff64864b5d4b1c041";
    let _not_matching= "0xf09500718fa31ffb89bc0374b95f2b1f39047b2e3e01058984a9697e045a94b3";
    let _not_matching2 = "0xb044ddc49d105964890f8e197c85f42d23737356015a07586a4f9237666526a8";

    let tx_hash_str = _type_2;
    let tx_hash = B256::from_str(tx_hash_str)?;

    let tx = provider
        .get_transaction_by_hash(tx_hash)
        .await?
        .ok_or("Transaction not found")?;

    let maybe_rx = provider
        .get_transaction_receipt(tx_hash)
        .await?;

    if let Some(rx) = maybe_rx {
        let solidity_packed = encoding::solidity_pack::solidity_packed_encode(tx.clone(), rx.clone())?;
    
        println!("tx:\n {:?}\n\nrx:\n{:?}\n\n", tx.clone(), rx.clone());
        //println!("types: {:?} abi: 0x{}", encoded.types, hex::encode(encoded.abi));
        println!("types: {:?}\n\nabi: 0x{}", solidity_packed.types, hex::encode(solidity_packed.abi));

    }   

    Ok(())
}

async fn _encode_block() -> Result<(), Box<dyn std::error::Error>> 
{
    let rpc_url = "https://sepolia-proxy-rpc.creditcoin.network";
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    //let block_number = BlockId::from(7846292);
    let block_number = BlockId::from(7853137);
    let block= provider
        .get_block(block_number, BlockTransactionsKind::Full)
        .await?;

    match block {
        Some(block) => {

            // get block receipts..
            let block_receipts = provider.get_block_receipts(block_number).await?;
            match block_receipts {
                Some(receipts) => {

                    // we have block and receipts now :)
                    let mut transaction_abis = Vec::new();
                    let mut transaction_packed_abi = Vec::new();
                    let mut transaction_safe_packed_abi = Vec::new();

                    let transactions = block.transactions.into_transactions_vec();
                    let transaction_and_receipts = transactions
                        .iter()
                        .zip(receipts);

                    for (tx, rx) in  transaction_and_receipts{
                        let encoded = abi_encode(tx.clone(), rx.clone())?;
                        let packed_encoded = encoding::solidity_pack::solidity_packed_encode(tx.clone(), rx.clone())?;
                        let safe_packed_encoded = encoding::safe_solidity_pack::safe_solidity_packed_encode(tx.clone(), rx.clone())?;
                        // transaction_abis.push(JsonAbiEncoded {
                        //     types: encoded.types,
                        //     abi: format!("0x{}", hex::encode(encoded.abi))
                        // });
                        transaction_abis.push(format!("0x{}", hex::encode(encoded.abi)).to_string());
                        transaction_packed_abi.push(format!("0x{}", hex::encode(packed_encoded.abi).to_string()));
                        transaction_safe_packed_abi.push(format!("0x{}", hex::encode(safe_packed_encoded.abi).to_string()));
                    }

                    let json_string = serde_json::to_string_pretty(&transaction_abis)?;
                    let mut file = File::create("../ignore/alloy-out/block.json")?;
                    file.write_all(json_string.as_bytes())?;

                    let json_string_packed = serde_json::to_string_pretty(&transaction_packed_abi)?;
                    let mut file = File::create("../ignore/alloy-out/solidity-packed-block.json")?;
                    file.write_all(json_string_packed.as_bytes())?;

                    let json_string_safed_packed = serde_json::to_string_pretty(&transaction_safe_packed_abi)?;
                    let mut file = File::create("../ignore/alloy-out/safe-solidity-packed-block.json")?;
                    file.write_all(json_string_safed_packed.as_bytes())?;
                },
                None => {
                    println!("Could not find receipts for block {:?}", block_number);
                }
            }
        },
        None => {
            println!("block {:?} not found", block_number);
        }
    }

    Ok(())
}

pub fn print_decoded_result(fields: Vec<FieldMetadata>, prefix: String) {
    for field in fields {
        println!("{}Type: {:?}, Offset: {}, size: {:?}, dynamic: {}, value: {}",
            prefix,
            field.sol_type,
            field.offset,
            field.size,
            field.is_dynamic,
            match field.value {
                Some(val) => {
                    format!("0x{}", hex::encode(val))
                },
                None => {
                    "N/A".into()
                }
            }
        );

        print_decoded_result(field.children.clone(), format!("{}\t", prefix));
    }
}

async fn _decoder_play() -> Result<(), Box<dyn std::error::Error>> 
{
    let values = vec![
        DynSolValue::Uint(U256::from(1), 8),
        DynSolValue::FixedArray(vec![
            DynSolValue::Uint(U256::from(2), 8),
            DynSolValue::Uint(U256::from(3), 8),
        ]),
        DynSolValue::Uint(U256::from(10), 256),
        DynSolValue::FixedArray(vec![
            DynSolValue::Bytes([3; 32].to_vec()),
            DynSolValue::Bytes([2; 32].to_vec()),
        ]),
        DynSolValue::Uint(U256::from(1), 256),
        DynSolValue::Bytes(vec![0, 1, 2]),
        DynSolValue::Bytes([[3; 32], [2; 32]].concat()),
        DynSolValue::Tuple(vec![
            DynSolValue::Bool(true),
            DynSolValue::Bool(false),
            DynSolValue::Uint(U256::from(1), 8),
            DynSolValue::Uint(U256::from(2), 64),
            DynSolValue::Uint(U256::from(3), 256)
        ]),
        DynSolValue::Tuple(vec![
            DynSolValue::Address(Address::ZERO),
            DynSolValue::Bytes(vec![0, 1, 2]),
            DynSolValue::Array(vec![
                DynSolValue::Uint(U256::from(1), 8),
                DynSolValue::Uint(U256::from(2), 8),
                DynSolValue::Uint(U256::from(3), 8),
            ])
        ]),
        DynSolValue::Array(vec![
            DynSolValue::Tuple(vec![
                DynSolValue::Address(Address::parse_checksummed("0x0000000000000000000000000000000000000001", None).unwrap()),
                DynSolValue::Bytes(vec![0, 1, 2]),
            ]),
            DynSolValue::Tuple(vec![
                DynSolValue::Address(Address::ZERO),
                DynSolValue::Bytes(vec![0, 1, 3]),
            ])
        ])
    ];

    let tuple = DynSolValue::Tuple(values.clone());
    match tuple.abi_encode_sequence() {
        Some(abi) => {
            // now that we have it encoded :)
            // lets call out custom decoder that returns ranges :>
            println!("ABI: 0x{}", hex::encode(abi.clone()));

            let types: Vec<DynSolType> = values.clone().iter().map(|value| {
                value.as_type().unwrap()
            }).collect();

            let result = compute_abi_offsets(types, abi);
           
            match result {
                Ok(decoded_result) => {
                    println!("\nABI Offsets:");
                    print_decoded_result(decoded_result, "".into());
                },
                Err(e) => {
                    println!("failed to compute offsets: {:?}", e);
                }
            }
        }, 
        None => {
            println!("failed to encode...");
        }
    };
    

    Ok(())
}

async fn query_builer_fun() -> Result<(), Box<dyn std::error::Error>>
{
    // RPC.
    let rpc_url = "https://sepolia-proxy-rpc.creditcoin.network";
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    // which transaction.
    let tx_hash_str = "0xc990ce703dd3ca83429c302118f197651678de359c271f205b9083d4aa333aae";
    let tx_hash = B256::from_str(tx_hash_str)?;

    // get the transaction & receipt.
    let tx = provider.get_transaction_by_hash(tx_hash).await?.unwrap();
    let rx = provider.get_transaction_receipt(tx_hash).await?.unwrap();

    // encode this transaction.
    let encoded = abi_encode(tx.clone(), rx.clone())?;
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
        },
    }

    let mut query_builder = QueryBuilder::create_from_transaction(tx, rx).expect("creating queryable builder should work");
    query_builder.set_abi_provider(Arc::new(|contract_address| {
        Box::pin(my_abi_provider(contract_address.clone()))
    }));

    query_builder.add_static_field(QueryableFields::RxStatus).expect("Should work to add from field");
    query_builder.add_static_field(QueryableFields::TxFrom).expect("Should work to add from field");
    query_builder.add_static_field(QueryableFields::TxTo).expect("Should work to add from field");

    // just to keep it simple for now
    // i'll just say i care about the event index 1
    // which is the transfer event from, to (zero address), amount
    query_builder.event_builder("Transfer".into(), 
        |log, event, log_index| {
            log_index == 1   
        },
        |builder| {

            // fun no? :)
            builder
                .add_address()?
                .add_signature()?
                .add_argument("from")?
                .add_argument("to")?
                .add_argument("value")?;

            Ok(())
        }
    ).await.expect("should have matched an event and constructed offsets");    

    query_builder.function_builder("burn".into(), |b| {
        b.add_signature()?.add_argument("value".into())?;
        Ok(())
    }).await.expect("should be able to query some calldata segments");
   
    println!("\nResult of query segments:");
    let selected_offsets = query_builder.get_selected_offsets();
    let raw = encoded.abi.clone();
    for (offset, size) in selected_offsets {
        let slice = &raw[offset..offset+size];
        println!("({}, {})\t 0x{}", offset, size, hex::encode(slice));
    }

    Ok(())
}

async fn multi_event_builder_fun() -> Result<(), Box<dyn std::error::Error>>
{
    // RPC.
    let rpc_url = "https://mainnet-proxy-rpc.creditcoin.network";
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    // which transaction.
    let tx_hash_str = "0xd746058082c13f30e2e219f78a150e536ccacf71b262e6fa5ce0f4b4a17a1cf6";
    let tx_hash = B256::from_str(tx_hash_str)?;

    // get the transaction & receipt.
    let tx = provider.get_transaction_by_hash(tx_hash).await?.unwrap();
    let rx = provider.get_transaction_receipt(tx_hash).await?.unwrap();

    // encode this transaction.
    let encoded = abi_encode(tx.clone(), rx.clone())?;
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
        },
    }

    let mut query_builder = QueryBuilder::create_from_transaction(tx, rx).expect("creating queryable builder should work");
    query_builder.set_abi_provider(Arc::new(|contract_address| {
        Box::pin(my_abi_provider(contract_address.clone()))
    }));

    query_builder.add_static_field(QueryableFields::RxStatus).expect("Should work to add from field");

    // in this case I care about all transfer events inside this transaction
    // the segments will be created in a sequential order
    query_builder.multi_event_builder("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef".into(), 
        |log, event, log_index| {
            true
        },
        |builder| {

            // fun no? :)
            builder
                .add_address()?
                .add_signature()?
                .add_argument("from")?
                .add_argument("to")?
                .add_argument("value")?;

            Ok(())
        }
    ).await.expect("should be able to query many events :)");
   
    println!("\nResult of query segments:");
    let selected_offsets = query_builder.get_selected_offsets();
    let raw = encoded.abi.clone();
    for (offset, size) in selected_offsets {
        let slice = &raw[offset..offset+size];
        println!("({}, {})\t 0x{}", offset, size, hex::encode(slice));
    }

    Ok(())
}

pub async fn my_abi_provider(contract_address: String) -> Option<String> {

    // hard coded G-CRE's ABI
    let json_str = r#"[{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales730Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"VestingStartDate","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"name","outputs":[{"name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"spender","type":"address"},{"name":"value","type":"uint256"}],"name":"approve","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"totalSupply","outputs":[{"name":"amount","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf365Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"value","type":"uint256"},{"name":"sighash","type":"string"}],"name":"exchange","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"from","type":"address"},{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transferFrom","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf183Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"decimals","outputs":[{"name":"","type":"uint8"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales1095Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf365Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"value","type":"uint256"}],"name":"burn","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf2190Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales183Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale365Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf730Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"owner","type":"address"}],"name":"balanceOf","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[],"name":"finalizeSales","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"from","type":"address"},{"name":"value","type":"uint256"}],"name":"burnFrom","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf2190Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf730Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"symbol","outputs":[{"name":"","type":"string"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale183Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"to","type":"address"},{"name":"value","type":"uint256"}],"name":"transfer","outputs":[{"name":"success","type":"bool"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"creditcoinSalesLimit","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"vestedBalanceOf1095Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"creditcoinLimitInFrac","outputs":[{"name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale2190Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale730Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf1095Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"name":"allowance","outputs":[{"name":"remaining","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[],"name":"startVesting","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales2190Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[{"name":"tokenHolder","type":"address"}],"name":"purchasedBalanceOf183Days","outputs":[{"name":"balance","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"IsSalesFinalized","outputs":[{"name":"","type":"bool"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolders","type":"address[]"},{"name":"amounts","type":"uint256[]"}],"name":"recordSales365Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"name":"tokenHolder","type":"address"},{"name":"numCoins","type":"uint256"}],"name":"recordSale1095Days","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"inputs":[{"name":"creditcoinFoundation","type":"address"},{"name":"devCost","type":"address"}],"payable":false,"stateMutability":"nonpayable","type":"constructor"},{"payable":true,"stateMutability":"payable","type":"fallback"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":false,"name":"value","type":"uint256"},{"indexed":true,"name":"sighash","type":"string"}],"name":"Exchange","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Burnt","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"from","type":"address"},{"indexed":true,"name":"to","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Transfer","type":"event"},{"anonymous":false,"inputs":[{"indexed":true,"name":"owner","type":"address"},{"indexed":true,"name":"spender","type":"address"},{"indexed":false,"name":"value","type":"uint256"}],"name":"Approval","type":"event"}]"#;

    Some(json_str.into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> 
{
    //_encode_transaction().await
    //_encode_block().await
    //decoder_play().await
    query_builer_fun().await
    //multi_event_builder_fun().await
}