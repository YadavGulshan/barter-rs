use crate::v2::engine::{
    state::{ConnectivityManager, EngineState, InstrumentStateManager},
    Processor,
};
use barter_data::event::MarketEvent;
use barter_instrument::exchange::ExchangeId;

impl<Kind, Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
    Processor<&MarketEvent<InstrumentKey, Kind>>
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
where
    Self: ConnectivityManager<ExchangeId> + InstrumentStateManager<InstrumentKey, Market = Market>,
    Market: for<'a> Processor<&'a MarketEvent<InstrumentKey, Kind>>,
    Strategy: for<'a> Processor<&'a MarketEvent<InstrumentKey, Kind>>,
    Risk: for<'a> Processor<&'a MarketEvent<InstrumentKey, Kind>>,
{
    type Audit = ProcessMarketStreamEventAudit;

    fn process(&mut self, event: &MarketEvent<InstrumentKey, Kind>) -> Self::Audit {
        // Todo: set exchange ConnectivityState to healthy if unhealthy

        self.instrument_mut(&event.instrument).market.process(event);
        self.strategy.process(event);
        self.risk.process(event);

        ProcessMarketStreamEventAudit
    }
}

pub struct ProcessMarketStreamEventAudit;
