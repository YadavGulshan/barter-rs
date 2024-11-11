use crate::v2::{
    engine::action::{
        close_positions::ClosePositionsOutput, generate_algo_orders::GenerateAlgoOrdersOutput,
        send_requests::SendRequestsOutput,
    },
    order::{RequestCancel, RequestOpen},
};
use derive_more::From;
use serde::{Deserialize, Serialize};

pub mod cancel_orders;
pub mod close_positions;
pub mod generate_algo_orders;
pub mod send_requests;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, From)]
pub enum ActionOutput<ExchangeKey, InstrumentKey> {
    GenerateAlgoOrders(GenerateAlgoOrdersOutput<ExchangeKey, InstrumentKey>),
    CancelOrders(SendRequestsOutput<ExchangeKey, InstrumentKey, RequestCancel>),
    OpenOrders(SendRequestsOutput<ExchangeKey, InstrumentKey, RequestOpen>),
    ClosePositions(ClosePositionsOutput<ExchangeKey, InstrumentKey>),
}
