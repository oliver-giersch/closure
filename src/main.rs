use std::error::Error;

use tokio::runtime::Runtime;
use web3::futures::StreamExt;

const PATH: &str = "~/.ethereum/bor.ipc";

fn main() -> Result<(), Box<dyn Error>> {
    let runtime = Runtime::new()?;
    Ok(runtime.block_on(event_loop())?)
}

async fn event_loop() -> Result<(), web3::Error> {
    let transport = web3::transports::Ipc::new(PATH).await?;
    let web3 = web3::Web3::new(transport);

    let mut stream = web3.eth_subscribe().subscribe_new_heads().await?;
    while let Some(Ok(block)) = stream.next().await {
        let number = block.number.unwrap().as_u64();
        println!("BLOCK EVENT #{}", number);
    }

    Err(web3::Error::Unreachable)
}
