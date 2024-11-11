use crate::v2::{
    engine::{
        execution_tx::ExecutionTxMap,
        state::{
            asset::AssetStates,
            instrument::{InstrumentState, InstrumentStates},
            EngineState, StateManager,
        },
        v2::{
            action::engine::{send_requests, SendRequestsOutput},
            Engine,
        },
    },
    order::{Order, RequestCancel, RequestOpen},
    risk::RiskManager,
};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait ClosePosition<ExchangeKey, InstrumentKey> {
    type Output;

    fn close_position(
        &mut self,
        txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>,
        instrument: &InstrumentKey,
    ) -> Self::Output;
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Constructor)]
pub struct ClosePositionOutput<ExchangeKey, InstrumentKey> {
    pub cancels: SendRequestsOutput<ExchangeKey, InstrumentKey, RequestCancel>,
    pub opens: SendRequestsOutput<ExchangeKey, InstrumentKey, RequestOpen>,
}

impl<MarketState, StrategyT, Risk, ExchangeKey, InstrumentKey, AssetKey>
    ClosePosition<ExchangeKey, InstrumentKey>
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
    StrategyT: ClosePositionStrategy<MarketState, AssetKey, ExchangeKey, InstrumentKey>,
    Risk: RiskManager<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    ExchangeKey: Debug + Clone,
    InstrumentKey: Debug + Clone,
    EngineState<MarketState, StrategyT::State, Risk::State, ExchangeKey, AssetKey, InstrumentKey>:
        StateManager<
            InstrumentKey,
            State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
        >,
{
    type Output = ClosePositionOutput<ExchangeKey, InstrumentKey>;

    fn close_position(
        &mut self,
        txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>,
        instrument: &InstrumentKey,
    ) -> Self::Output {
        // Generate orders
        let (cancels, opens) = self.strategy.close_position_request(
            instrument,
            &self.state.strategy,
            &self.state.assets,
            &self.state.instruments,
        );

        // Bypass risk checks...

        // Send order requests
        let cancels = send_requests(txs, cancels);
        let opens = send_requests(txs, opens);

        // Record in flight order requests
        self.state.record_in_flight_cancels(cancels.sent.iter());
        self.state.record_in_flight_opens(opens.sent.iter());

        ClosePositionOutput::new(cancels, opens)
    }
}
pub trait ClosePositionStrategy<MarketState, AssetKey, ExchangeKey, InstrumentKey> {
    type State;
    fn close_position_request(
        &self,
        instrument: &InstrumentKey,
        strategy_state: &Self::State,
        asset_states: &AssetStates,
        instrument_states: &InstrumentStates<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    ) -> (
        impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestCancel>>,
        impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestOpen>>,
    );
}
