use crate::v2::engine::{
    state::{process::trading_state::ProcessTradingStateAudit, EngineState},
    Processor,
};
use serde::{Deserialize, Serialize};

pub trait TradingStateManager
where
    Self: Processor<TradingState, Audit = ProcessTradingStateAudit>,
{
    fn trading_state(&self) -> TradingState;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum TradingState {
    Enabled,
    Disabled,
}

impl<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey> TradingStateManager
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
{
    fn trading_state(&self) -> TradingState {
        self.trading
    }
}
