use crate::v2::{
    engine::{
        error::EngineError,
        execution_tx::ExecutionTxMap,
        state::{instrument::InstrumentState, EngineState, StateManager},
        v2::{
            action::engine::{send_request, SendRequestsOutput},
            Engine,
        },
    },
    order::{Order, RequestCancel},
    risk::RiskManager,
    strategy::Strategy,
};
use barter_integration::collection::none_one_or_many::NoneOneOrMany;
use std::fmt::Debug;

pub trait CancelOrder<ExchangeKey, InstrumentKey> {
    type Output;

    fn cancel_order(
        &mut self,
        txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>,
        request: Order<ExchangeKey, InstrumentKey, RequestCancel>,
    ) -> Self::Output;
}

impl<MarketState, StrategyT, Risk, ExchangeKey, InstrumentKey, AssetKey>
    CancelOrder<ExchangeKey, InstrumentKey>
    for Engine<
        EngineState<
            MarketState,
            StrategyT::State,
            Risk::State,
            ExchangeKey,
            AssetKey,
            InstrumentKey,
        >,
        StrategyT,
        Risk,
    >
where
    StrategyT: Strategy<MarketState, AssetKey, ExchangeKey, InstrumentKey>,
    Risk: RiskManager<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    ExchangeKey: Debug + Clone,
    InstrumentKey: Debug + Clone,
    EngineState<MarketState, StrategyT::State, Risk::State, ExchangeKey, AssetKey, InstrumentKey>:
        StateManager<
            InstrumentKey,
            State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
        >,
{
    type Output = SendRequestsOutput<ExchangeKey, InstrumentKey, RequestCancel>;

    fn cancel_order(
        &mut self,
        txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>,
        request: Order<ExchangeKey, InstrumentKey, RequestCancel>,
    ) -> Self::Output {
        // Bypass risk checks...

        // Send order request
        match send_request(txs, &request) {
            Ok(_) => SendRequestsOutput::new(NoneOneOrMany::One(request), NoneOneOrMany::None),
            Err(error) => SendRequestsOutput::new(
                NoneOneOrMany::None,
                NoneOneOrMany::One((request, EngineError::from(error))),
            ),
        }
    }
}
