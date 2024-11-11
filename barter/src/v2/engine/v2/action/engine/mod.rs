use crate::v2::{
    engine::{
        audit::AuditEvent,
        error::{EngineError, UnrecoverableEngineError},
        execution_tx::ExecutionTxMap,
    },
    execution::ExecutionRequest,
    order::Order,
};
use barter_integration::{channel::Tx, collection::none_one_or_many::NoneOneOrMany, Unrecoverable};
use derive_more::Constructor;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use tracing::error;

pub mod cancel_all_orders;
pub mod cancel_order;
pub mod cancel_order_by_id;
pub mod close_all_positions;
pub mod close_position;
pub mod generate_algo_orders;
pub mod open_order;

// Perhaps just Engine "action" methods should be in the new pattern, States can just be event
// processor

pub trait Auditor<Kind> {
    fn send_audit(&mut self, audit: AuditEvent<Kind>);
}

// fn run<Events, Engine, AuditKind>(
//     feed: &mut Events,
//     engine: &mut Engine,
//     auditor: &mut impl Auditor<AuditKind>
// ) -> AuditEvent<AuditKind>
// where
//     Events: Iterator,
//     Events::Item: Clone,
//     Engine: Processor<Events::Item>,
//     Engine::Output: Into<AuditKind>,
// {
//     // Send initial Engine state snapshot
//     // auditor.send_audit()
//
//     // Todo: this appears to be much the same, apart from we now have an Auditor, which I think is
//     //   a good idea.
//
//
// }

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

pub fn send_requests<ExchangeKey, InstrumentKey, Kind>(
    txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>,
    requests: impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, Kind>>,
) -> SendRequestsOutput<ExchangeKey, InstrumentKey, Kind>
where
    ExchangeKey: Debug + Clone,
    InstrumentKey: Debug + Clone,
    Kind: Debug + Clone,
    ExecutionRequest<ExchangeKey, InstrumentKey>: From<Order<ExchangeKey, InstrumentKey, Kind>>,
{
    // Send order requests
    let (sent, errors): (Vec<_>, Vec<_>) = requests
        .into_iter()
        .map(|request| {
            send_request(txs, &request)
                .map_err(|error| (request.clone(), EngineError::from(error)))
                .map(|_| request)
        })
        .partition_result();

    SendRequestsOutput::new(NoneOneOrMany::from(sent), NoneOneOrMany::from(errors))
}

pub fn send_request<ExchangeKey, InstrumentKey, Kind>(
    txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>,
    request: &Order<ExchangeKey, InstrumentKey, Kind>,
) -> Result<(), UnrecoverableEngineError>
where
    ExchangeKey: Debug + Clone,
    InstrumentKey: Debug + Clone,
    Kind: Debug + Clone,
    ExecutionRequest<ExchangeKey, InstrumentKey>: From<Order<ExchangeKey, InstrumentKey, Kind>>,
{
    match txs
        .find(&request.exchange)?
        .send(ExecutionRequest::from(request.clone()))
    {
        Ok(()) => Ok(()),
        Err(error) if error.is_unrecoverable() => {
            Err(UnrecoverableEngineError::ExecutionChannelTerminated(
                format!(
                    "{:?} execution channel terminated, failed to send {:?}",
                    request.exchange, request
                )
                .to_string(),
            ))
        }
        Err(error) => {
            error!(
                exchange = ?request.exchange,
                ?request,
                ?error,
                "failed to send ExecutionRequest due to unhealthy channel"
            );
            Ok(())
        }
    }
}
