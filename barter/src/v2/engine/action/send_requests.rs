use crate::v2::{
    engine::error::EngineError
    ,
    order::Order,
};
use barter_integration::{channel::Tx, collection::none_one_or_many::NoneOneOrMany, Unrecoverable};
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

// pub fn send_requests<ExchangeKey, InstrumentKey, Kind>(
//     txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>,
//     requests: impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, Kind>>,
// ) -> SendRequestsOutput<ExchangeKey, InstrumentKey, Kind>
// where
//     ExchangeKey: Debug + Clone,
//     InstrumentKey: Debug + Clone,
//     Kind: Debug + Clone,
//     ExecutionRequest<ExchangeKey, InstrumentKey>: From<Order<ExchangeKey, InstrumentKey, Kind>>,
// {
//     // Send order requests
//     let (sent, errors): (Vec<_>, Vec<_>) = requests
//         .into_iter()
//         .map(|request| {
//             send_request(txs, &request)
//                 .map_err(|error| (request.clone(), EngineError::from(error)))
//                 .map(|_| request)
//         })
//         .partition_result();
//
//     SendRequestsOutput::new(NoneOneOrMany::from(sent), NoneOneOrMany::from(errors))
// }
//
// pub fn send_request<ExchangeKey, InstrumentKey, Kind>(
//     txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>,
//     request: &Order<ExchangeKey, InstrumentKey, Kind>,
// ) -> Result<(), UnrecoverableEngineError>
// where
//     ExchangeKey: Debug + Clone,
//     InstrumentKey: Debug + Clone,
//     Kind: Debug + Clone,
//     ExecutionRequest<ExchangeKey, InstrumentKey>: From<Order<ExchangeKey, InstrumentKey, Kind>>,
// {
//     match txs
//         .find(&request.exchange)?
//         .send(ExecutionRequest::from(request.clone()))
//     {
//         Ok(()) => Ok(()),
//         Err(error) if error.is_unrecoverable() => {
//             Err(UnrecoverableEngineError::ExecutionChannelTerminated(
//                 format!(
//                     "{:?} execution channel terminated, failed to send {:?}",
//                     request.exchange, request
//                 )
//                 .to_string(),
//             ))
//         }
//         Err(error) => {
//             // Todo: this does not actualy send the request, but would be ack'd
//             error!(
//                 exchange = ?request.exchange,
//                 ?request,
//                 ?error,
//                 "failed to send ExecutionRequest due to unhealthy channel"
//             );
//             Ok(())
//         }
//     }
// }
