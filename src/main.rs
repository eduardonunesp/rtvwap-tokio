mod trade;
mod trade_feed;
mod trade_pair;
mod trade_provider;
mod trade_providers;
mod vwap;

use tokio::sync::mpsc;
use trade_pair::{Currency, TradePair};
use trade_providers::coinbase::CoinbaseProvider;
use vwap::{VWAPResult, VWAP};

fn create_vwap_result_chan(
    buffer_size: usize,
) -> (mpsc::Sender<VWAPResult>, mpsc::Receiver<VWAPResult>) {
    mpsc::channel(buffer_size)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (tx, mut rx) = create_vwap_result_chan(32);

    let trade_groups = vec![(
        TradePair::new(Currency::BTC, Currency::USD),
        CoinbaseProvider::new(),
    )];

    for (pair, provider) in trade_groups {
        let trade_feeder = trade_feed::new_trade_feed(pair, provider).await?;
        let tx = tx.clone();

        // Run calculation in a separate task
        tokio::spawn(async move {
            let mut vwap = VWAP::new(trade_feeder.trade_provider.rx);
            vwap.calculate(tx).await;
        });
    }

    // Run the main loop 100 times
    for _ in 0..100 {
        tokio::select! {
            vwap_result = rx.recv() => {
                if let Some(vwap_result) = vwap_result {
                    let to: &str = vwap_result.pair.to.into();
                    let from: &str = vwap_result.pair.from.into();
                    // Precise value to .2
                    println!("VWAP: from {} to {} value {:.2}", from, to, vwap_result.vwap_value);
                }
            }
        }
    }

    Ok(())
}
