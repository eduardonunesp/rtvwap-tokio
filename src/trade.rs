use crate::trade_pair::TradePair;

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
