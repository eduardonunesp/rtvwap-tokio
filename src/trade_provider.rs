use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::{trade::Trade, trade_pair::TradePair};

#[derive(Debug)]
pub struct TradeProvider {
    pub tx: mpsc::Sender<Trade>,
    pub rx: mpsc::Receiver<Trade>,
}

impl TradeProvider {
    pub fn new() -> TradeProvider {
        let (tx, rx) = mpsc::channel::<Trade>(32);

        TradeProvider { tx, rx }
    }
}

#[async_trait]
pub trait TradeProviderCreator {
    async fn create_trade_provider(&self, pair: TradePair) -> Result<TradeProvider, anyhow::Error>;
}
