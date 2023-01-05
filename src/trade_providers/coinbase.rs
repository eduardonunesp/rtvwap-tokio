use anyhow::Ok;
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Message, Result},
    MaybeTlsStream, WebSocketStream,
};
use url::Url;

use crate::{
    trade::Trade, trade_pair::TradePair, trade_provider::TradeProvider,
    trade_provider::TradeProviderCreator,
};

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

#[derive(Serialize, Deserialize)]
struct SubscriptionRes {
    #[serde(rename = "type")]
    ttype: String,
}

impl CoinbaseProvider {
    async fn wait_subscription(
        &self,
        ws_stream: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
        pair: TradePair,
    ) -> Result<(), anyhow::Error> {
        let sub_req = SubReq {
            ttype: "subscribe".to_string(),
            product_ids: vec![pair.to_string()],
            channels: vec!["matches".to_string()],
        };

        ws_stream
            .send(Message::Text(serde_json::to_string(&sub_req)?))
            .await?;

        while let Some(msg) = ws_stream.next().await {
            let msg = msg.unwrap();
            let msg = serde_json::from_str::<SubscriptionRes>(&msg.to_string()).unwrap();

            if msg.ttype == "subscriptions" {
                break;
            }
        }

        println!("Subscribed to {}", pair.to_string());

        Ok(())
    }

    async fn wait_trade(
        &self,
        tp_tx: tokio::sync::mpsc::Sender<Trade>,
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        pair: TradePair,
    ) -> Result<(), anyhow::Error> {
        let mut ws_stream = ws_stream;

        tokio::spawn(async move {
            while let Some(msg) = ws_stream.next().await {
                let msg = msg.unwrap();
                let msg = serde_json::from_str::<MatchRes>(&msg.to_string()).unwrap();

                if msg.ttype == "match" {
                    let price = msg.price.parse::<f64>().unwrap();
                    let quantity = msg.size.parse::<f64>().unwrap();

                    if price == 0.0 || quantity == 0.0 {
                        continue;
                    }

                    tp_tx
                        .send(Trade::new(pair.clone(), price, quantity))
                        .await
                        .unwrap();
                }
            }
        });

        Ok(())
    }
}

#[async_trait]
impl TradeProviderCreator for CoinbaseProvider {
    async fn create_trade_provider(&self, pair: TradePair) -> Result<TradeProvider, anyhow::Error> {
        let trade_provider = TradeProvider::new();

        let url = Url::parse(WS_URL)?;

        let (mut ws_stream, _) = connect_async(url).await?;

        self.wait_subscription(&mut ws_stream, pair.clone()).await?;

        let tp_tx = trade_provider.tx.clone();

        self.wait_trade(tp_tx, ws_stream, pair.clone()).await?;

        Ok(trade_provider)
    }
}

#[cfg(test)]
mod tests {
    use crate::trade_pair::Currency;

    use super::*;
    use std::{sync::Arc, sync::Mutex, time::Duration};
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_coinbase_provider() {
        let coinbase_provider = CoinbaseProvider {};
        let trade_pair = TradePair::new(Currency::BTC, Currency::USD);
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
