use std::{fs::File, io::Write, str::FromStr};
use alloy::{eips::BlockId, primitives::B256, providers::{Provider, ProviderBuilder}, rpc::types::BlockTransactionsKind};
use encoding::abi::abi_encode;
use serde::Serialize;


mod encoding;

#[derive(Serialize)]
struct JsonAbiEncoded(Vec<String>, String);

async fn encode_transaction() -> Result<(), Box<dyn std::error::Error>> {

    let rpc_url = "https://sepolia-proxy-rpc.creditcoin.network";
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    let _type_3 = "0x085d2fe01372711005b053a1b0d081c13cde19b6ddb77cae847e0d11a0a0cafe";
    let _type_2 = "0xdfba59b94bac3da5af5d0fa8b81ae3199069fa6f38002be58c14e94a051e0642";
    let _legacy = "0x0b50111d729c00bac4a99702b2c88e425321c8f8214bc3272072c730d5ff9ad2";
    let _type_4 = "0x2dce846c932bcf50a9dd180e61a09818009da734f2e6761cf9e825f19077f05a";
    let _type_1 = "0x5c8c6d8c61bd8109ce02717db62b12554c097d156b66e30ff64864b5d4b1c041";
    let _not_matching= "0xf09500718fa31ffb89bc0374b95f2b1f39047b2e3e01058984a9697e045a94b3";

    let tx_hash_str = _type_4;
    let tx_hash = B256::from_str(tx_hash_str)?;

    let tx = provider
        .get_transaction_by_hash(tx_hash)
        .await?
        .ok_or("Transaction not found")?;

    let maybe_rx = provider
        .get_transaction_receipt(tx_hash)
        .await?;

    if let Some(rx) = maybe_rx {
        ///let encoded = encoding::abi::abi_encode(tx.clone(), rx.clone())?;
        let solidity_packed = encoding::solidity_pack::solidity_packed_encode(tx.clone(), rx.clone())?;
    
        println!("tx:\n {:?}\n\nrx:\n{:?}", tx.clone(), rx.clone());
        //println!("types: {:?} abi: 0x{}", encoded.types, hex::encode(encoded.abi));
        println!("types: {:?} abi: 0x{}", solidity_packed.types, hex::encode(solidity_packed.abi));

    }   

    Ok(())
}

async fn encode_block() -> Result<(), Box<dyn std::error::Error>> 
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

                    let transactions = block.transactions.into_transactions_vec();
                    let transaction_and_receipts = transactions
                        .iter()
                        .zip(receipts);

                    for (tx, rx) in  transaction_and_receipts{
                        let encoded = abi_encode(tx.clone(), rx.clone())?;
                        let packed_encoded = encoding::solidity_pack::solidity_packed_encode(tx.clone(), rx.clone())?;
                        // transaction_abis.push(JsonAbiEncoded {
                        //     types: encoded.types,
                        //     abi: format!("0x{}", hex::encode(encoded.abi))
                        // });
                        transaction_abis.push(format!("0x{}", hex::encode(encoded.abi)).to_string());
                        transaction_packed_abi.push(format!("0x{}", hex::encode(packed_encoded.abi).to_string()));
                    }

                    let json_string = serde_json::to_string_pretty(&transaction_abis)?;
                    let mut file = File::create("../ignore/alloy-out/block.json")?;
                    file.write_all(json_string.as_bytes())?;

                    let json_string2 = serde_json::to_string_pretty(&transaction_packed_abi)?;
                    let mut file = File::create("../ignore/alloy-out/solidity-packed-block.json")?;
                    file.write_all(json_string2.as_bytes())?;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> 
{
    //encode_transaction().await
    encode_block().await
}