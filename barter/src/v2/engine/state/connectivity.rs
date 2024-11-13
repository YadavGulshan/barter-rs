use barter_instrument::exchange::ExchangeId;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

// I'm feeling pretty committed to abstracting state updates with trait, but not trait EngineState,
// or Processor<'a> rather one for each "update"

// pub trait ConnectivityStateManager<ExchangeKey> {
//     fn state(
//         &self,
//         key: &ExchangeKey
//     ) ->
// }

#[derive(Debug, Clone, Eq, PartialEq, Default, Deserialize, Serialize)]
pub struct ConnectivityStates(pub IndexMap<ExchangeId, ConnectivityState>);

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Deserialize, Serialize)]
pub struct ConnectivityState {
    pub market_data: Connection,
    pub account: Connection,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Deserialize, Serialize)]
pub enum Connection {
    Healthy,
    Reconnecting,
}

impl Default for Connection {
    fn default() -> Self {
        Self::Healthy
    }
}
