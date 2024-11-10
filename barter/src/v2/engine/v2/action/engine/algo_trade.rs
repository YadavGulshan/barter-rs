use crate::v2::{
    engine::{
        action::RecordOrdersInFlight,
        audit::request::{
            risk_refused::RiskRefusedRequestsAudit, sent::SentRequestsAudit, ExecutionRequestAudit,
        },
        error::{EngineError, UnrecoverableEngineError},
        execution_tx::ExecutionTxMap,
        state::EngineState,
        v2::Engine,
    },
    execution::ExecutionRequest,
    order::{Order, RequestCancel, RequestOpen},
    risk::{RiskApproved, RiskManager},
    strategy::Strategy,
};
use barter_integration::{channel::Tx, collection::none_one_or_many::NoneOneOrMany, Unrecoverable};
use itertools::Itertools;
use std::fmt::Debug;
use tracing::error;

pub trait AlgoTrade<ExchangeKey, InstrumentKey> {
    type Output;
    fn trade(&mut self, txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>) -> Self::Output;
}

pub type SentOrderRequests<ExchangeKey, InstrumentKey> =
    ExecutionRequestAudit<ExchangeKey, InstrumentKey>;

impl<MarketState, StrategyT, Risk, ExchangeKey, AssetKey, InstrumentKey>
    AlgoTrade<ExchangeKey, InstrumentKey>
    for Engine<
        EngineState<
            MarketState,
            StrategyT::State,
            Risk::State,
            ExchangeKey,
            AssetKey,
            InstrumentKey,
        >,
        StrategyT,
        Risk,
    >
where
    Self: RecordOrdersInFlight<ExchangeKey, InstrumentKey>,
    StrategyT: Strategy<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    Risk: RiskManager<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    ExchangeKey: Debug + Clone,
    InstrumentKey: Debug + Clone,
{
    type Output = SentOrderRequests<ExchangeKey, InstrumentKey>;

    fn trade(&mut self, txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>) -> Self::Output {
        // Generate orders
        let (cancels, opens) = self.strategy.generate_orders(
            &self.state.strategy,
            &self.state.assets,
            &self.state.instruments,
        );

        // RiskApprove & RiskRefuse order requests
        let (cancels, opens, refused_cancels, refused_opens) = self.risk.check(
            &self.state.risk,
            &self.state.assets,
            &self.state.instruments,
            cancels,
            opens,
        );

        // Send risk approved order requests
        let sent = send_orders(
            txs,
            cancels.into_iter().map(|RiskApproved(cancel)| cancel),
            opens.into_iter().map(|RiskApproved(open)| open),
        );

        // Collect remaining Iterators (so we can &mut self)
        let refused = RiskRefusedRequestsAudit {
            refused_cancels: refused_cancels.into_iter().collect(),
            refused_opens: refused_opens.into_iter().collect(),
        };

        // Record in flight order requests
        self.record_requests_in_flight(&sent.cancels, &sent.opens);

        ExecutionRequestAudit { sent, refused }
    }
}

pub fn send_orders<ExchangeKey, InstrumentKey>(
    txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>,
    cancels: impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestCancel>>,
    opens: impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestOpen>>,
) -> SentRequestsAudit<ExchangeKey, InstrumentKey>
where
    ExchangeKey: Debug + Clone,
    InstrumentKey: Debug + Clone,
{
    // Send order requests
    let (cancels_sent, cancel_send_errs): (Vec<_>, Vec<_>) = cancels
        .into_iter()
        .map(|cancel| {
            send_order(txs, &cancel)
                .map_err(|error| (cancel.clone(), EngineError::from(error)))
                .map(|_| cancel)
        })
        .partition_result();

    let (opens_sent, open_send_errs): (Vec<_>, Vec<_>) = opens
        .into_iter()
        .map(|open| {
            send_order(txs, &open)
                .map_err(|error| (open.clone(), EngineError::from(error)))
                .map(|_| open)
        })
        .partition_result();

    SentRequestsAudit {
        cancels: NoneOneOrMany::Many(cancels_sent),
        opens: NoneOneOrMany::Many(opens_sent),
        failed_cancels: NoneOneOrMany::from(cancel_send_errs),
        failed_opens: NoneOneOrMany::from(open_send_errs),
    }
}

pub fn send_order<ExchangeKey, InstrumentKey, Kind>(
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
