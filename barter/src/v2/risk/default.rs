use crate::v2::{
    engine::{state::EngineState, Processor},
    execution::AccountEvent,
    order::{Order, RequestCancel, RequestOpen},
    risk::{RiskApproved, RiskManager, RiskRefused},
};
use barter_data::event::MarketEvent;
use std::marker::PhantomData;

/// Example [`RiskManager`] implementation that approves all order requests.
///
/// *EXAMPLE IMPLEMENTATION ONLY, PLEASE DO NOT USE FOR ANYTHING OTHER THAN TESTING PURPOSES.*
#[derive(Debug, Clone)]
pub struct DefaultRiskManager<Market, Strategy, AssetKey> {
    phantom_data: PhantomData<(Market, Strategy, AssetKey)>,
}

impl<ExchangeKey, InstrumentKey, Market, Strategy, AssetKey> RiskManager<ExchangeKey, InstrumentKey>
    for DefaultRiskManager<Market, Strategy, AssetKey>
where
    Market: Clone,
    Strategy: Clone,
    ExchangeKey: Clone,
    AssetKey: Clone,
    InstrumentKey: Clone,
{
    type State = EngineState<
        Market,
        Strategy,
        DefaultRiskManagerState,
        ExchangeKey,
        AssetKey,
        InstrumentKey,
    >;

    fn check(
        &self,
        _: &Self::State,
        cancels: impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestCancel>>,
        opens: impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestOpen>>,
    ) -> (
        impl IntoIterator<Item = RiskApproved<Order<ExchangeKey, InstrumentKey, RequestCancel>>>,
        impl IntoIterator<Item = RiskApproved<Order<ExchangeKey, InstrumentKey, RequestOpen>>>,
        impl IntoIterator<Item = RiskRefused<Order<ExchangeKey, InstrumentKey, RequestCancel>>>,
        impl IntoIterator<Item = RiskRefused<Order<ExchangeKey, InstrumentKey, RequestOpen>>>,
    ) {
        (
            cancels.into_iter().map(RiskApproved::new),
            opens.into_iter().map(RiskApproved::new),
            std::iter::empty(),
            std::iter::empty(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct DefaultRiskManagerState;

impl<ExchangeKey, AssetKey, InstrumentKey>
    Processor<&AccountEvent<ExchangeKey, AssetKey, InstrumentKey>> for DefaultRiskManagerState
{
    type Audit = ();
    fn process(&mut self, _: &AccountEvent<ExchangeKey, AssetKey, InstrumentKey>) -> Self::Audit {}
}

impl<InstrumentKey, Kind> Processor<&MarketEvent<InstrumentKey, Kind>> for DefaultRiskManagerState {
    type Audit = ();
    fn process(&mut self, _: &MarketEvent<InstrumentKey, Kind>) -> Self::Audit {}
}
