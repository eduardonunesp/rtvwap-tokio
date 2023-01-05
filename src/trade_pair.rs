use strum_macros::{EnumString, IntoStaticStr};

#[derive(EnumString, IntoStaticStr, Debug, Clone, PartialEq, Eq)]
pub enum Currency {
    #[strum(serialize = "USD")]
    USD,
    #[strum(serialize = "BTC")]
    BTC,
}

#[derive(Debug, Clone)]
pub struct TradePair {
    pub from: Currency,
    pub to: Currency,
}

impl TradePair {
    pub fn new(from: Currency, to: Currency) -> TradePair {
        TradePair { from, to }
    }

    pub fn to_string(&self) -> String {
        let from: &str = self.from.clone().into();
        let to: &str = self.to.clone().into();
        format!("{}-{}", from, to)
    }
}
