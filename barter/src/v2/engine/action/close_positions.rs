use crate::v2::{
    engine::{
        action::send_requests::{send_requests, SendRequestsOutput},
        command::InstrumentFilter,
        execution_tx::ExecutionTxMap,
        state::{instrument::InstrumentState, StateManager},
        Engine,
    },
    order::{Order, RequestCancel, RequestOpen},
    risk::RiskManager,
};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait ClosePositions<ExchangeKey, InstrumentKey> {
    type Output;

    fn close_positions(
        &mut self,
        filter: &InstrumentFilter<ExchangeKey, InstrumentKey>,
    ) -> Self::Output;
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Constructor)]
pub struct ClosePositionsOutput<ExchangeKey, InstrumentKey> {
    pub cancels: SendRequestsOutput<ExchangeKey, InstrumentKey, RequestCancel>,
    pub opens: SendRequestsOutput<ExchangeKey, InstrumentKey, RequestOpen>,
}

impl<State, MarketState, ExecutionTxs, Strategy, Risk, ExchangeKey, InstrumentKey, AssetKey>
    ClosePositions<ExchangeKey, InstrumentKey> for Engine<State, ExecutionTxs, Strategy, Risk>
where
    State: StateManager<
        InstrumentKey,
        State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    >,
    ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
    Strategy: CloseAllPositionsStrategy<ExchangeKey, InstrumentKey, State = State>,
    ExchangeKey: Debug + Clone,
    InstrumentKey: Debug + Clone,
{
    type Output = ClosePositionsOutput<ExchangeKey, InstrumentKey>;

    fn close_positions(
        &mut self,
        filter: &InstrumentFilter<ExchangeKey, InstrumentKey>,
    ) -> Self::Output {
        // Generate orders
        let (cancels, opens) = self.strategy.close_positions_requests(&filter, &self.state);

        // Bypass risk checks...

        // Send order requests
        let cancels = send_requests(&self.execution_txs, cancels);
        let opens = send_requests(&self.execution_txs, opens);

        // Record in flight order requests
        self.record_in_flight_cancels(&cancels.sent);
        self.record_in_flight_opens(&opens.sent);

        ClosePositionsOutput::new(cancels, opens)
    }
}

pub trait CloseAllPositionsStrategy<ExchangeKey, InstrumentKey> {
    type State;
    fn close_positions_requests(
        &self,
        filter: &InstrumentFilter<ExchangeKey, InstrumentKey>,
        state: &Self::State,
    ) -> (
        impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestCancel>>,
        impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestOpen>>,
    );
}

// impl<MarketState, ExecutionTxs, StrategyT, Risk, ExchangeKey, InstrumentKey, AssetKey>
//     ClosePositions<ExchangeKey, InstrumentKey>
//     for Engine<
//         EngineState<
//             MarketState,
//             StrategyT::State,
//             Risk::State,
//             ExchangeKey,
//             AssetKey,
//             InstrumentKey,
//         >,
//         ExecutionTxs,
//         StrategyT,
//         Risk,
//     >
// where
//     ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
//     StrategyT: CloseAllPositionsStrategy<MarketState, AssetKey, ExchangeKey, InstrumentKey>,
//     Risk: RiskManager<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
//     ExchangeKey: Debug + Clone,
//     InstrumentKey: Debug + Clone,
//     EngineState<MarketState, StrategyT::State, Risk::State, ExchangeKey, AssetKey, InstrumentKey>:
//         StateManager<
//             InstrumentKey,
//             State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
//         >,
// {
//     type Output = ClosePositionsOutput<ExchangeKey, InstrumentKey>;
//
//     fn close_positions(
//         &mut self,
//         filter: InstrumentFilter<ExchangeKey, InstrumentKey>,
//     ) -> Self::Output {
//         // Generate orders
//         let (cancels, opens) = self.strategy.close_positions_requests(
//             filter,
//             &self.state.strategy,
//             &self.state.assets,
//             &self.state.instruments,
//         );
//
//         // Bypass risk checks...
//
//         // Send order requests
//         let cancels = send_requests(&self.execution_txs, cancels);
//         let opens = send_requests(&self.execution_txs, opens);
//
//         // Record in flight order requests
//         self.state.record_in_flight_cancels(cancels.sent.iter());
//         self.state.record_in_flight_opens(opens.sent.iter());
//
//         ClosePositionsOutput::new(cancels, opens)
//     }
// }

// pub trait CloseAllPositionsStrategy<MarketState, AssetKey, ExchangeKey, InstrumentKey> {
//     type State;
//     fn close_positions_requests(
//         &self,
//         filter: InstrumentFilter<ExchangeKey, InstrumentKey>,
//         strategy_state: &Self::State,
//         asset_states: &AssetStates,
//         instrument_states: &InstrumentStates<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
//     ) -> (
//         impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestCancel>>,
//         impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestOpen>>,
//     );
// }
