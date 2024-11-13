use crate::v2::engine::{state::EngineState, Processor};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum TradingState {
    Enabled,
    Disabled,
}

pub trait TradingStateManager
where
    Self: Processor<TradingState, Audit = ProcessTradingStateAudit>,
{
    fn state(&self) -> TradingState;
}

impl<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey> Processor<TradingState>
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
{
    type Audit = ProcessTradingStateAudit;

    fn process(&mut self, event: TradingState) -> Self::Audit {
        let prev = self.trading;
        let next = match (self.trading, event) {
            (TradingState::Enabled, TradingState::Disabled) => {
                info!("EngineState setting TradingState::Disabled");
                TradingState::Disabled
            }
            (TradingState::Disabled, TradingState::Enabled) => {
                info!("EngineState setting TradingState::Enabled");
                TradingState::Enabled
            }
            (TradingState::Enabled, TradingState::Enabled) => {
                info!("EngineState set TradingState::Enabled, although it was already enabled");
                TradingState::Enabled
            }
            (TradingState::Disabled, TradingState::Disabled) => {
                info!("EngineState set TradingState::Disabled, although it was already disabled");
                TradingState::Disabled
            }
        };

        self.trading = next;

        ProcessTradingStateAudit {
            prev,
            current: next,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub struct ProcessTradingStateAudit {
    pub prev: TradingState,
    pub current: TradingState,
}

impl<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey> TradingStateManager
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
{
    fn state(&self) -> TradingState {
        self.trading
    }
}
