use crate::v2::engine::state::EngineState;
use serde::{Deserialize, Serialize};
use tracing::info;

pub trait TradingStateUpdater {
    type Audit;
    fn update_from_trading_state_update(&mut self, event: TradingState) -> Self::Audit;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum TradingState {
    Enabled,
    Disabled,
}

pub struct TradingStateUpdaterAudit {
    pub prev: TradingState,
    pub current: TradingState,
}

impl<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey> TradingStateUpdater
for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
{
    type Audit = Option<TradingStateUpdaterAudit>;

    fn update_from_trading_state_update(&mut self, event: TradingState) -> Self::Audit {
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

        if prev != next {
            self.trading = next;
            Some(TradingStateUpdaterAudit {
                prev,
                current: self.trading,
            })
        } else {
            None
        }
    }
}