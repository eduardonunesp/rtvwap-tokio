use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, Result},
};
use url::Url;

use crate::trade::{Trade, TradePair, TradeProvider, TradeProviderCreator};

const WS_URL: &str = "wss://ws-feed.exchange.coinbase.com";

pub struct CoinbaseProvider {}

impl CoinbaseProvider {
    pub fn new() -> CoinbaseProvider {
        CoinbaseProvider {}
    }
}

#[derive(Serialize, Deserialize)]
struct SubReq {
    #[serde(rename = "type")]
    ttype: String,
    product_ids: Vec<String>,
    channels: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct MatchRes {
    #[serde(rename = "type")]
    ttype: String,
    trade_id: u64,
    maker_order_id: String,
    taker_order_id: String,
    side: String,
    size: String,
    price: String,
    product_id: String,
    sequence: u64,
    time: String,
}

#[async_trait]
impl TradeProviderCreator for CoinbaseProvider {
    async fn create_trade_provider(
        &self,
        pair: TradePair,
    ) -> Result<TradeProvider, Box<dyn std::error::Error>> {
        let trade_provider = TradeProvider::new();

        let url = Url::parse(WS_URL)?;

        let (mut ws_stream, _) = connect_async(url).await?;

        let sub_req = SubReq {
            ttype: "subscribe".to_string(),
            product_ids: vec![pair.to_string()],
            channels: vec!["matches".to_string()],
        };

        ws_stream
            .send(Message::Text(serde_json::to_string(&sub_req)?))
            .await?;

        ws_stream.next().await;

        let tp_tx = trade_provider.tx.clone();

        tokio::spawn(async move {
            while let Some(msg) = ws_stream.next().await {
                let msg = msg.unwrap();
                let msg: MatchRes = serde_json::from_str(&msg.to_string()).unwrap();

                let trade = Trade::new(
                    pair.clone(),
                    msg.price.parse().unwrap(),
                    msg.size.parse().unwrap(),
                );

                tp_tx.send(trade.clone()).await.unwrap();
            }
        });

        Ok(trade_provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::Arc, sync::Mutex, time::Duration};
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_coinbase_provider() {
        let coinbase_provider = CoinbaseProvider {};
        let trade_pair = TradePair::new("BTC".to_string(), "USD".to_string());
        let mut trade_provider = coinbase_provider
            .create_trade_provider(trade_pair)
            .await
            .unwrap();

        let count = Arc::new(Mutex::from(0));

        let c = Arc::clone(&count);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = trade_provider.rx.recv() => {
                        let mut v = c.lock().unwrap();
                        *v += 1;
                    }
                    _ = sleep(Duration::from_secs(1)) => {
                        continue
                    }
                }
            }
        });

        sleep(Duration::from_secs(3)).await;
        assert!(*count.lock().unwrap() > 0);
    }
}
