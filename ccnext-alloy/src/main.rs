use std::str::FromStr;
use alloy::{primitives::B256, providers::{Provider, ProviderBuilder}};


mod encoding;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> 
{
    let rpc_url = "https://sepolia-proxy-rpc.creditcoin.network";
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    let type_3 = "0x085d2fe01372711005b053a1b0d081c13cde19b6ddb77cae847e0d11a0a0cafe";
    let type_2 = "0xdfba59b94bac3da5af5d0fa8b81ae3199069fa6f38002be58c14e94a051e0642";
    let legacy = "0x0b50111d729c00bac4a99702b2c88e425321c8f8214bc3272072c730d5ff9ad2";
    let type_4 = "0x2dce846c932bcf50a9dd180e61a09818009da734f2e6761cf9e825f19077f05a";
    let type_1 = "0xabb6c72501a70bb25f5d8a96c638547632814d1f3891c4c4b8fde87cf88eec59";

    let tx_hash_str = type_1;
    let tx_hash = B256::from_str(tx_hash_str)?;

    let tx = provider
        .get_transaction_by_hash(tx_hash)
        .await?
        .ok_or("Transaction not found")?;

    let maybe_rx = provider
        .get_transaction_receipt(tx_hash)
        .await?;

    if let Some(rx) = maybe_rx {
        let encoded = encoding::abi::abi_encode(tx.clone(), rx.clone());
        match encoded {
            Ok(result) => {
                println!("tx:\n {:?}\n\nrx:\n{:?}", tx.clone(), rx.clone());
                println!("types: {:?} abi: 0x{}", result.types, hex::encode(result.abi));
            },
            Err(err) => {
                println!("failed to encode transaction {:?}", err);
            }
        }
      
    }   

    Ok(())
}