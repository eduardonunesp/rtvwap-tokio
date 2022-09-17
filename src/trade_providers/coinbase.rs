use tokio::sync::mpsc;

use crate::trade::{TradePair, TradeProviderCreator, TradeProvider, Trade};

pub struct CoinbaseProvider {

}

impl TradeProviderCreator for CoinbaseProvider {
    fn createTradeProvider(&self, pair: TradePair) -> Result<TradeProvider, Box<dyn std::error::Error>> {
        let trade_provider = TradeProvider::new();

        let (tx, rx) = mpsc::channel::<Trade>(32);

        let tradeProvider = TradeProvider {
            txTradeChan : tx,
            rxTradeChan : rx,
        };

        Ok(tradeProvider)
    }
}