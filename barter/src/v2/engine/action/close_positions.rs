use crate::v2::{
    engine::{
        action::send_requests::SendRequestsOutput, command::InstrumentFilter,
        execution_tx::ExecutionTxMap, Engine, InstrumentStateManager,
    },
    order::{Order, RequestCancel, RequestOpen},
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

impl<State, ExecutionTxs, Strategy, Risk, ExchangeKey, InstrumentKey>
    ClosePositions<ExchangeKey, InstrumentKey> for Engine<State, ExecutionTxs, Strategy, Risk>
where
    State: InstrumentStateManager<InstrumentKey, Exchange = ExchangeKey>,
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
        let (cancels, opens) = self.strategy.close_positions_requests(filter, &self.state);

        // Bypass risk checks...

        // Send order requests
        let cancels = self.send_requests(cancels);
        let opens = self.send_requests(opens);

        // Record in flight order requests
        self.state.record_in_flight_cancels(&cancels.sent);
        self.state.record_in_flight_opens(&opens.sent);

        ClosePositionsOutput::new(cancels, opens)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Constructor)]
pub struct ClosePositionsOutput<ExchangeKey, InstrumentKey> {
    pub cancels: SendRequestsOutput<ExchangeKey, InstrumentKey, RequestCancel>,
    pub opens: SendRequestsOutput<ExchangeKey, InstrumentKey, RequestOpen>,
}
