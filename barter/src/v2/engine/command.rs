use crate::v2::order::{Order, RequestCancel, RequestOpen};
use barter_integration::collection::one_or_many::OneOrMany;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum Command<ExchangeKey, InstrumentKey> {
    SendCancelRequests(OneOrMany<Order<ExchangeKey, InstrumentKey, RequestCancel>>),
    SendOpenRequests(OneOrMany<Order<ExchangeKey, InstrumentKey, RequestOpen>>),
    ClosePositions(InstrumentFilter<ExchangeKey, InstrumentKey>),
    CancelOrders(InstrumentFilter<ExchangeKey, InstrumentKey>),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum InstrumentFilter<ExchangeKey, InstrumentKey> {
    None,
    Exchanges(OneOrMany<ExchangeKey>),
    Instruments(OneOrMany<InstrumentKey>),
}
