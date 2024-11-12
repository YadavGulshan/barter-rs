use crate::v2::engine::state::{EngineState, StateManager};
use crate::v2::execution::AccountEvent;
use barter_data::streams::reconnect;
use crate::v2::engine::state::connectivity::ConnectivityState;
use crate::v2::engine::state::update::account_event::AccountEventUpdater;

pub trait ReconnectEventUpdater<ExchangeKey, Kind> {
    type Audit;
    fn update_from_reconnect(&mut self, event: reconnect::Event<ExchangeKey, Kind>);
}

impl<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
ReconnectEventUpdater<ExchangeKey, AccountEvent<ExchangeKey, AssetKey, InstrumentKey>>
for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
where
    Self: AccountEventUpdater<ExchangeKey, AssetKey, InstrumentKey>,
    Self: StateManager<ExchangeKey, State = ConnectivityState>,
{
    type Audit = ();

    fn update_from_reconnect(
        &mut self, 
        event: reconnect::Event<ExchangeKey, AccountEvent<ExchangeKey, AssetKey, InstrumentKey>>
    ) {
        match 
    }
}