use async_trait::async_trait;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct TradePair {
    pub left: String,
    pub right: String,
}

impl TradePair {
    pub fn new(left: String, right: String) -> TradePair {
        TradePair { left, right }
    }

    pub fn to_string(&self) -> String {
        format!("{}-{}", self.left, self.right)
    }
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub trade_pair: TradePair,
    pub price: f64,
    pub quantity: f64,
}

impl Trade {
    pub fn new(trade_pair: TradePair, price: f64, quantity: f64) -> Trade {
        Trade {
            trade_pair,
            price,
            quantity,
        }
    }
}

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
    async fn create_trade_provider(
        &self,
        pair: TradePair,
    ) -> Result<TradeProvider, Box<dyn std::error::Error>>;
}

pub struct TradeFeed {
    pub trade_provider_chan: TradeProvider,
}

pub async fn new_trade_feed(
    left: String,
    right: String,
    trade_provider: Box<dyn TradeProviderCreator>,
) -> Result<TradeFeed, Box<dyn std::error::Error>> {
    return new_trade_feed_with_pair(TradePair { left, right }, trade_provider).await;
}

pub async fn new_trade_feed_with_pair(
    pair: TradePair,
    trade_provider: Box<dyn TradeProviderCreator>,
) -> Result<TradeFeed, Box<dyn std::error::Error>> {
    let trade_feed = TradeFeed {
        trade_provider_chan: trade_provider.create_trade_provider(pair).await?,
    };

    Ok(trade_feed)
}
