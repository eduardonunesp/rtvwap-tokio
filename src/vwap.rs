use std::time::Duration;

use tokio::{select, sync::mpsc, time::sleep};

use crate::{trade::Trade, trade_pair::TradePair};

const QUEUE_BUFFER_SIZE: usize = 200;

#[derive(Debug)]
pub struct VWAPResult {
    pub pair: TradePair,
    pub vwap_value: f64,
}

impl VWAPResult {
    pub fn new(pair: TradePair, vwap_value: f64) -> VWAPResult {
        VWAPResult { pair, vwap_value }
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

    pub async fn calculate(&mut self, vwap_result: mpsc::Sender<VWAPResult>) {
        loop {
            select! {
                trade = self.trade_chan.recv() => {
                    match trade {
                        Some(trade) => {
                            self.trade_samples.push(trade.clone());

                            if self.trade_samples.len() > QUEUE_BUFFER_SIZE {
                                self.trade_samples.remove(0);
                            }

                            let mut sum_price_and_volume: f64 = 0.;

                            for t in self.trade_samples.iter() {
                                let price_and_volume = t.price * t.quantity;
                                sum_price_and_volume = sum_price_and_volume + price_and_volume;
                            }

                            let mut sum_volume: f64 = 0.;

                            for t in self.trade_samples.iter() {
                                sum_volume = sum_volume + t.quantity;
                            }

                            let result = VWAPResult::new(
                                trade.trade_pair,
                                sum_price_and_volume / sum_volume,
                            );

                            vwap_result.send(result).await.unwrap();
                        },
                        None => {
                            println!("Trade channel closed");
                        }
                    }
                },

                _ = sleep(Duration::from_secs(1)) => {
                    continue;
                }

            }
        }
    }
}
