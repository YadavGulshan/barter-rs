use crate::v2::{engine::error::EngineError, order::Order};
use barter_integration::{collection::none_one_or_many::NoneOneOrMany, Unrecoverable};
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Constructor)]
pub struct SendRequestsOutput<ExchangeKey, InstrumentKey, Kind> {
    pub sent: NoneOneOrMany<Order<ExchangeKey, InstrumentKey, Kind>>,
    pub errors: NoneOneOrMany<(Order<ExchangeKey, InstrumentKey, Kind>, EngineError)>,
}

impl<ExchangeKey, InstrumentKey, Kind> Unrecoverable
    for SendRequestsOutput<ExchangeKey, InstrumentKey, Kind>
{
    fn is_unrecoverable(&self) -> bool {
        self.errors
            .iter()
            .any(|(_, error)| error.is_unrecoverable())
    }
}
