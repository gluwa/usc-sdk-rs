use std::str::FromStr;
use alloy::{primitives::B256, providers::{Provider, ProviderBuilder}};


mod encoding;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> 
{
    let rpc_url = "https://sepolia-proxy-rpc.creditcoin.network";
    let provider = ProviderBuilder::new().on_http(rpc_url.parse()?);

    let tx_hash_str = "0x0b50111d729c00bac4a99702b2c88e425321c8f8214bc3272072c730d5ff9ad2"; //"0xe4e2d78020e382f20e68445b624e17182b98f47c612a6587adf634e6195e2f65";
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