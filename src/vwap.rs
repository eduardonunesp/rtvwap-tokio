use num_bigint::BigInt;
use tokio::{sync::mpsc, select};

use crate::trade::{TradePair, Trade};

pub struct VWAPResult {
    pub pair: TradePair,
    pub vwap_value: BigInt,
}

impl VWAPResult {
    pub fn new(pair: TradePair) -> VWAPResult {
        VWAPResult {
            pair,
            vwap_value: BigInt::from(0),
        }
    }
}

pub struct VWAP {
    pub trade_chan: mpsc::Receiver<Trade>,
    trade_samples: Vec<Trade>,
}

impl VWAP {
    pub fn new(trade_chan: mpsc::Receiver<Trade>) -> VWAP {
        VWAP {
            trade_chan,
            trade_samples: Vec::new(),
        }
    }

    pub async fn calculate(&mut self, vwapResult: &mut mpsc::Sender<VWAPResult>) {
        select! {
            trade = self.trade_chan.recv() => {
                match trade {
                    Some(trade) => {
                        self.trade_samples.push(trade);

                        let mut sum_price_and_volume = BigInt::from(0);

                        for trade in self.trade_samples.iter() {
                            let price_and_volume = trade.price.clone() * trade.quantity.clone();
                            sum_price_and_volume = sum_price_and_volume.clone() + price_and_volume.clone();
                        }

                        let mut sum_volume = BigInt::from(0);

                        for trade in self.trade_samples.iter() {
                            sum_volume = sum_volume.clone() + trade.quantity.clone();
                        }

                        let result = VWAPResult{
                            pair: TradePair::new("BTC".to_string(), "USDT".to_string()),
                            vwap_value: sum_price_and_volume.clone() + sum_volume.clone(),
                        };

                        vwapResult.send(result).await;
                    },
                    None => {
                        println!("Trade channel closed");
                    }
                }
            },
        }
    }
}