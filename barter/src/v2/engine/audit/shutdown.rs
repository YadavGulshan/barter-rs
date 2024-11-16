use crate::v2::engine::{audit::Audit, error::UnrecoverableEngineError};
use barter_integration::collection::one_or_many::OneOrMany;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum ShutdownAudit<Event> {
    FeedEnded,
    Error(Event, OneOrMany<UnrecoverableEngineError>),
    Commanded(Event),
}

pub enum ShutdownAuditNew {
    Command
}

impl<State, Event, Output> From<ShutdownAudit<Event>> for Audit<State, Event, Output> {
    fn from(value: ShutdownAudit<Event>) -> Self {
        Self::Shutdown(value)
    }
}
