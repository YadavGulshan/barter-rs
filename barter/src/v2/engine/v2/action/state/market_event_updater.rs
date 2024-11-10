use crate::v2::engine::{
    state::{
        connectivity::{Connection, ConnectivityState},
        instrument::InstrumentState,
        EngineState, StateManager,
    },
    Processor,
};
use barter_data::{
    event::{DataKind, MarketEvent},
    streams::consumer::MarketStreamEvent,
};
use barter_instrument::exchange::ExchangeId;
use tracing::warn;

pub trait MarketEventUpdater<InstrumentKey, Kind> {
    type Audit;
    fn update_from_market(&mut self, event: &MarketStreamEvent<InstrumentKey, Kind>);
}

pub struct MarketEventUpdaterAudit;

impl<InstrumentKey, Kind, Market, Strategy, Risk, ExchangeKey, AssetKey>
    MarketEventUpdater<InstrumentKey, Kind>
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
where
    Self: StateManager<ExchangeId, State = ConnectivityState>
        + StateManager<
            InstrumentKey,
            State = InstrumentState<Market, ExchangeKey, AssetKey, InstrumentKey>,
        >,
    Market: for<'a> Processor<&'a MarketEvent<InstrumentKey, Kind>>,
    Strategy: for<'a> Processor<&'a MarketEvent<InstrumentKey, Kind>>,
    Risk: for<'a> Processor<&'a MarketEvent<InstrumentKey, Kind>>,
{
    type Audit = MarketEventUpdaterAudit;

    fn update_from_market(&mut self, event: &MarketStreamEvent<InstrumentKey, Kind>) {
        match event {
            MarketStreamEvent::Reconnecting(exchange) => {
                warn!(
                    ?exchange,
                    "EngineState received MarketStream disconnected event"
                );
                self.state_mut(exchange).market_data = Connection::Reconnecting;
            }
            MarketStreamEvent::Item(event) => {
                // Todo: set exchange ConnectivityState to healthy if unhealthy
                self.state_mut(&event.instrument).market.process(event);
                self.strategy.process(event);
                self.risk.process(event);
            }
        }
    }
}
