use crate::v2::engine::{
    state::{connectivity::Connection, EngineState},
    InstrumentStateManager, Processor,
};
use barter_data::{event::MarketEvent, streams::consumer::MarketStreamEvent};
use tracing::warn;

impl<InstrumentKey, Kind, Market, Strategy, Risk, ExchangeKey, AssetKey>
    Processor<&MarketStreamEvent<InstrumentKey, Kind>>
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
where
    Self: InstrumentStateManager<InstrumentKey>,
    Market: for<'a> Processor<&'a MarketEvent<InstrumentKey, Kind>>,
    Strategy: for<'a> Processor<&'a MarketEvent<InstrumentKey, Kind>>,
    Risk: for<'a> Processor<&'a MarketEvent<InstrumentKey, Kind>>,
{
    type Audit = ProcessMarketStreamEventAudit;

    fn process(&mut self, event: &MarketStreamEvent<InstrumentKey, Kind>) -> Self::Audit {
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
        };

        ProcessMarketStreamEventAudit
    }
}

pub struct ProcessMarketStreamEventAudit;
