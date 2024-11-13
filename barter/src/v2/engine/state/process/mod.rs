use crate::v2::{
    engine::{state::EngineState, Processor},
    execution::manager::AccountStreamEvent,
};
use barter_data::streams::consumer::MarketStreamEvent;

impl<InstrumentKey, Kind, Market, Strategy, Risk, ExchangeKey, AssetKey>
    Processor<MarketStreamEvent<InstrumentKey, Kind>>
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
{
    type Audit = ProcessMarketStreamEventAudit;

    fn process(&mut self, _: MarketStreamEvent<InstrumentKey, Kind>) -> Self::Audit {
        todo!()
    }
}

pub struct ProcessMarketStreamEventAudit;

impl<ExchangeKey, AssetKey, InstrumentKey, Market, Strategy, Risk>
    Processor<AccountStreamEvent<ExchangeKey, AssetKey, InstrumentKey>>
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
{
    type Audit = ProcessAccountStreamEventAudit;

    fn process(
        &mut self,
        _: AccountStreamEvent<ExchangeKey, AssetKey, InstrumentKey>,
    ) -> Self::Audit {
        todo!()
    }
}

pub struct ProcessAccountStreamEventAudit;
