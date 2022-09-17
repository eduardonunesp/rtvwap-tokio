use num_bigint::BigInt;
use tokio::sync::mpsc;


#[derive(Debug)]
pub struct TradePair {
    pub left : String,
    pub right : String,
}

impl TradePair {
    pub fn new(left : String, right : String) -> TradePair {
        TradePair {
            left,
            right,
        }
    }
}

#[derive(Debug)]
pub struct Trade {
    pub pairPair : TradePair,
    pub price : BigInt,
    pub quantity : BigInt,
}

impl Trade {
    pub fn new(pairPair : TradePair, price : BigInt, quantity : BigInt) -> Trade {
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
    pub txTradeChan : mpsc::Sender<Trade>,
    pub rxTradeChan : mpsc::Receiver<Trade>,
}

impl TradeProvider {
    pub fn new() -> TradeProvider {
        let (tx, rx) = mpsc::channel::<Trade>(32);

        TradeProvider {
            txTradeChan : tx,
            rxTradeChan : rx,
        }
    }
}

pub trait TradeProviderCreator {
    fn createTradeProvider(&self, pair: TradePair) -> Result<TradeProvider, Box<dyn std::error::Error>>;
}

pub struct TradeFeed {
    pub tradeProviderChan : TradeProvider
}

pub fn newTradeFeed(left: String, right: String, tradeProvider: Box<dyn TradeProviderCreator>) -> Result<TradeFeed, Box<dyn std::error::Error>> {
    return newTradeFeedWithPair(TradePair { left, right }, tradeProvider)
}

pub fn newTradeFeedWithPair(pair: TradePair, tradeProvider: Box<dyn TradeProviderCreator>) -> Result<TradeFeed, Box<dyn std::error::Error>> {
    let tradeFeed = TradeFeed{
        tradeProviderChan: tradeProvider.createTradeProvider(pair)?,
    };

    Ok(tradeFeed)
}
