mod trade;
mod vwap;
mod trade_providers;

use trade::{TradePair, TradeProvider};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let trade_pair = TradePair::new("BTC".to_string(), "USDT".to_string());
    // let trade = Trade::new(trade_pair, 100.into(), 100.into());
    // println!("{:?}", trade);
    // let mut vwap_result_chan = mpsc::channel

    let trade_feed_pairs = vec![
        (TradePair::new("BTC".to_string(), "USDT".to_string()), TradeProvider::new()),
    ];

    for (pair, provider) in trade_feed_pairs {
        println!("{:#?}", pair);
        println!("{:#?}", provider);
    }

    Ok(())
}
