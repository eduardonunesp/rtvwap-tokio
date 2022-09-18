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
    pub pairPair: TradePair,
    pub price: f64,
    pub quantity: f64,
}

impl Trade {
    pub fn new(pairPair: TradePair, price: f64, quantity: f64) -> Trade {
        let (tx, mut rx) = mpsc::channel::<i32>(32);

        Trade {
            pairPair,
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
    async fn createTradeProvider(
        &self,
        pair: TradePair,
    ) -> Result<TradeProvider, Box<dyn std::error::Error>>;
}

pub struct TradeFeed {
    pub tradeProviderChan: TradeProvider,
}

pub async fn newTradeFeed(
    left: String,
    right: String,
    tradeProvider: Box<dyn TradeProviderCreator>,
) -> Result<TradeFeed, Box<dyn std::error::Error>> {
    return newTradeFeedWithPair(TradePair { left, right }, tradeProvider).await;
}

pub async fn newTradeFeedWithPair(
    pair: TradePair,
    tradeProvider: Box<dyn TradeProviderCreator>,
) -> Result<TradeFeed, Box<dyn std::error::Error>> {
    let tradeFeed = TradeFeed {
        tradeProviderChan: tradeProvider.createTradeProvider(pair).await?,
    };

    Ok(tradeFeed)
}
