use crate::{
    trade_pair::TradePair,
    trade_provider::{TradeProvider, TradeProviderCreator},
};

pub struct TradeFeed {
    pub trade_provider: TradeProvider,
}

pub async fn new_trade_feed<T>(
    trade_pair: TradePair,
    trade_provider: T,
) -> Result<TradeFeed, anyhow::Error>
where
    T: TradeProviderCreator + Send + Sync + 'static,
{
    let trade_feed = TradeFeed {
        trade_provider: trade_provider.create_trade_provider(trade_pair).await?,
    };

    Ok(trade_feed)
}
