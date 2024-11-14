use crate::v2::{
    engine::{
        action::{
            cancel_orders::CancelOrders,
            close_positions::{ClosePositions, ClosePositionsOutput, ClosePositionsStrategy},
            generate_algo_orders::GenerateAlgoOrdersOutput,
            send_requests::SendRequestsOutput,
        },
        command::Command,
        execution_tx::ExecutionTxMap,
        state::order_manager::InFlightRequestRecorder,
        Engine,
    },
    order::{RequestCancel, RequestOpen},
};
use derive_more::From;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod cancel_orders;
pub mod close_positions;
pub mod generate_algo_orders;
pub mod send_requests;

impl<State, ExecutionTxs, Strategy, Risk> Engine<State, ExecutionTxs, Strategy, Risk> {
    pub fn action<ExchangeKey, InstrumentKey>(
        &mut self,
        command: &Command<ExchangeKey, InstrumentKey>,
    ) -> ActionOutput<ExchangeKey, InstrumentKey>
    where
        State: InFlightRequestRecorder<ExchangeKey, InstrumentKey>,
        ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
        Strategy: ClosePositionsStrategy<ExchangeKey, InstrumentKey, State = State>,
        ExchangeKey: Debug + Clone + PartialEq,
        InstrumentKey: Debug + Clone + PartialEq,
    {
        match &command {
            Command::SendCancelRequests(requests) => {
                let output = self.send_requests(requests.clone());
                self.state.record_in_flight_cancels(&output.sent);
                ActionOutput::CancelOrders(output)
            }
            Command::SendOpenRequests(requests) => {
                let output = self.send_requests(requests.clone());
                self.state.record_in_flight_opens(&output.sent);
                ActionOutput::OpenOrders(output)
            }
            Command::ClosePositions(filter) => {
                ActionOutput::ClosePositions(self.close_positions(filter))
            }
            Command::CancelOrders(filter) => ActionOutput::CancelOrders(self.cancel_orders(filter)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, From)]
pub enum ActionOutput<ExchangeKey, InstrumentKey> {
    GenerateAlgoOrders(GenerateAlgoOrdersOutput<ExchangeKey, InstrumentKey>),
    CancelOrders(SendRequestsOutput<ExchangeKey, InstrumentKey, RequestCancel>),
    OpenOrders(SendRequestsOutput<ExchangeKey, InstrumentKey, RequestOpen>),
    ClosePositions(ClosePositionsOutput<ExchangeKey, InstrumentKey>),
}
