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

    let tx_hash_str = type_3;
    let tx_hash = B256::from_str(tx_hash_str)?;

    let tx = provider
        .get_transaction_by_hash(tx_hash)
        .await?
        .ok_or("Transaction not found")?;

    let maybe_rx = provider
        .get_transaction_receipt(tx_hash)
        .await?;

    if let Some(rx) = maybe_rx {
        let encoded = encoding::abi::abi_encode(tx, rx);
        match encoded {
            Ok(result) => {
                println!("types: {:?} abi: 0x{}", result.types, hex::encode(result.abi));
            },
            Err(err) => {
                println!("failed to encode transaction {:?}", err);
            }
        }
      
    }   

    Ok(())
}