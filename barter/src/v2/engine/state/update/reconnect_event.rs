// use crate::v2::engine::state::connectivity::ConnectivityState;
// use crate::v2::engine::state::update::account_event::AccountEventUpdater;
// use crate::v2::engine::state::{EngineState, StateManager};
// use crate::v2::execution::AccountEvent;
// use barter_data::event::MarketEvent;
// use barter_data::streams::reconnect;
//
// pub trait ReconnectEventUpdater<ExchangeKey, Kind> {
//     type Audit;
//     fn update_from_reconnect(&mut self, event: &reconnect::Event<ExchangeKey, Kind>);
// }
//
// impl<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
// ReconnectEventUpdater<ExchangeKey, AccountEvent<ExchangeKey, AssetKey, InstrumentKey>>
// for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
// where
//     Self: AccountEventUpdater<ExchangeKey, AssetKey, InstrumentKey>,
//     Self: StateManager<ExchangeKey, State = ConnectivityState>,
// {
//     type Audit = ();
//
//     fn update_from_reconnect(
//         &mut self,
//         event: reconnect::Event<ExchangeKey, AccountEvent<ExchangeKey, AssetKey, InstrumentKey>>
//     ) {
//         todo!()
//     }
// }
//
// // May make sense to hold the "connectivity state" inside InstrumentState, but still have
// // reconnect::Event be keyed on ExchangeId. In the rare event there is a disconnection we can do a
// // regular lookup...
//
// impl<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
// ReconnectEventUpdater<ExchangeKey, MarketEvent<InstrumentKey>>
// for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
// where
//     Self: AccountEventUpdater<ExchangeKey, AssetKey, InstrumentKey>,
//     Self: StateManager<ExchangeKey, State = ConnectivityState>,
// {
//     type Audit = ();
//
//     fn update_from_reconnect(
//         &mut self,
//         event: reconnect::Event<ExchangeKey, AccountEvent<ExchangeKey, AssetKey, InstrumentKey>>
//     ) {
//         todo!()
//     }
// }
