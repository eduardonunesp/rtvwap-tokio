use std::{str::FromStr, time::Duration};

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt, TryStreamExt};
use num_bigint::{BigInt, ToBigInt};
use serde::{Deserialize, Serialize};
use tokio::{select, sync::mpsc, time::sleep};
use tokio_tungstenite::{
    connect_async, connect_async_tls_with_config,
    tungstenite::{Error, Message, Result},
};
use url::Url;

use crate::trade::{Trade, TradePair, TradeProvider, TradeProviderCreator};

pub struct CoinbaseProvider {}

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
    async fn createTradeProvider(
        &self,
        pair: TradePair,
    ) -> Result<TradeProvider, Box<dyn std::error::Error>> {
        let trade_provider = TradeProvider::new();

        let url = Url::parse("wss://ws-feed.exchange.coinbase.com")
            .expect("Can't connect to case count URL");

        let (mut ws_stream, _) = connect_async(url).await?;

        let sub_req = SubReq {
            ttype: "subscribe".to_string(),
            product_ids: vec![pair.to_string()],
            channels: vec!["matches".to_string()],
        };

        let sub_req = serde_json::to_string(&sub_req)?;

        ws_stream.send(Message::Text(sub_req)).await?;

        ws_stream.next().await;

        let tx = trade_provider.tx.clone();

        tokio::spawn(async move {
            while let Some(msg) = ws_stream.next().await {
                let msg = msg.unwrap();
                let msg: MatchRes = serde_json::from_str(&msg.to_string()).unwrap();

                let trade = Trade::new(
                    pair.clone(),
                    msg.price.parse().unwrap(),
                    msg.size.parse().unwrap(),
                );
                tx.send(trade.clone()).await;
            }
        });

        Ok(trade_provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_coinbase_provider() {
        let coinbase_provider = CoinbaseProvider {};
        let trade_pair = TradePair::new("BTC".to_string(), "USD".to_string());
        let mut trade_provider = coinbase_provider
            .createTradeProvider(trade_pair)
            .await
            .unwrap();

        println!("Trade provider created");

        tokio::spawn(async move {
            loop {
                println!("Waiting for trade");
                tokio::select! {
                    trade = trade_provider.rx.recv() => {
                        println!("{:?}", trade.unwrap());
                    }
                    _ = sleep(Duration::from_secs(1)) => {
                        continue
                    }
                }
            }
        });

        sleep(Duration::from_secs(10)).await;
        // println!("{:#?}", trade_provider);
    }
}
