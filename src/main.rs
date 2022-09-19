#![allow(warnings)]

mod trade;
mod trade_providers;
mod vwap;

use std::{
    borrow::BorrowMut,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::{sync::mpsc, time::sleep};
use trade::{new_trade_feed_with_pair, TradePair, TradeProvider};
use trade_providers::coinbase::CoinbaseProvider;
use vwap::{VWAPResult, VWAP};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, mut rx) = mpsc::channel::<VWAPResult>(100);

    let trade_feed_pairs = vec![(
        TradePair::new("BTC".to_string(), "USDT".to_string()),
        Box::new(CoinbaseProvider::new()),
    )];

    let mut vwaps: Arc<Mutex<Vec<VWAP>>> = Arc::new(Mutex::new(Vec::new()));

    for (pair, provider) in trade_feed_pairs {
        let trade_feeder = new_trade_feed_with_pair(pair, provider).await?;
        let tx = tx.clone();
        let v = vwaps.clone();
        tokio::spawn(async move {
            let mut vwap = VWAP::new(trade_feeder.trade_provider_chan.rx);
            vwap.calculate(tx).await;
            v.lock().unwrap().push(vwap);
        });
    }

    loop {
        tokio::select! {
            vwap_result = rx.recv() => {
                if let Some(vwap_result) = vwap_result {
                    println!("VWAP: {}", vwap_result.vwap_value);
                }
            }
        }
    }

    Ok(())
}
