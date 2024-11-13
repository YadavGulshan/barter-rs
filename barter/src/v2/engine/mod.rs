use crate::v2::{
    engine::{
        action::{
            cancel_orders::CancelOrders,
            close_positions::{CloseAllPositionsStrategy, ClosePositions},
            send_requests::SendRequestsOutput,
            ActionOutput,
        },
        audit::{AuditEvent, Auditor},
        command::{Command, InstrumentFilter},
        error::{EngineError, RecoverableEngineError, UnrecoverableEngineError},
        execution_tx::ExecutionTxMap,
        state::{
            asset::AssetState,
            instrument::{market_data::MarketDataState, InstrumentState},
            order_manager::OrderManager,
            trading_state_manager::TradingStateManager,
            EngineState, IndexedEngineState, StateManager,
        },
    },
    execution::{manager::AccountStreamEvent, ExecutionRequest},
    order::{Order, RequestCancel, RequestOpen},
    risk::RiskManager,
    strategy::Strategy,
    EngineEvent,
};
use audit::shutdown::ShutdownAudit;
use barter_data::streams::consumer::MarketStreamEvent;
use barter_instrument::{
    asset::{name::AssetNameInternal, AssetIndex},
    exchange::{ExchangeId, ExchangeIndex},
    instrument::{name::InstrumentNameInternal, InstrumentIndex},
};
use barter_integration::{
    channel::{ChannelTxDroppable, Tx},
    collection::none_one_or_many::NoneOneOrMany,
    Unrecoverable,
};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use std::fmt::Debug;
use tracing::error;

pub mod action;
pub mod audit;
pub mod command;
pub mod error;
pub mod execution_tx;
pub mod state;

pub trait Processor<Event> {
    type Audit;
    fn process(&mut self, event: Event) -> Self::Audit;
}

pub fn run<Events, Engine, AuditTx>(
    feed: &mut Events,
    engine: &mut Engine,
    audit_tx: &mut ChannelTxDroppable<AuditTx>,
) -> ShutdownAudit<Events::Item>
where
    Events: Iterator,
    Events::Item: Clone,
    Engine: Processor<Events::Item> + Auditor<Engine::Audit>,
    Engine::Audit: From<Engine::Snapshot> + From<ShutdownAudit<Events::Item>>,
    AuditTx: Tx<Item = AuditEvent<Engine::Audit>>,
{
    // Send initial Engine state snapshot
    audit_tx.send(engine.build_audit(engine.snapshot()));

    // Run Engine process loop until shutdown
    let shutdown_audit = loop {
        let Some(event) = feed.next() else {
            break ShutdownAudit::FeedEnded;
        };

        let audit_kind = engine.process(event);
        audit_tx.send(engine.build_audit(audit_kind));
    };

    // Send Shutdown audit
    audit_tx.send(engine.build_audit(shutdown_audit.clone()));
    shutdown_audit
}

pub struct Engine<State, ExecutionTxs, Strategy, Risk> {
    pub sequence: u64,
    pub clock: fn() -> DateTime<Utc>,
    pub state: State,
    pub execution_txs: ExecutionTxs,
    pub strategy: Strategy,
    pub risk: Risk,
    // pub actual_state:
}

impl<
        MarketEventKind,
        ExchangeKey,
        AssetKey,
        InstrumentKey,
        State,
        ExecutionTxs,
        Strategy,
        Risk,
    > Processor<EngineEvent<MarketEventKind, ExchangeKey, AssetKey, InstrumentKey>>
    for Engine<State, ExecutionTxs, Strategy, Risk>
where
    State: InstrumentStateManager<InstrumentKey, Exchange = ExchangeKey>
        + TradingStateManager
        + for<'a> Processor<&'a MarketStreamEvent<InstrumentKey, MarketEventKind>>
        + for<'a> Processor<&'a AccountStreamEvent<ExchangeKey, AssetKey, InstrumentKey>>,
    ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
    Strategy: CloseAllPositionsStrategy<ExchangeKey, InstrumentKey, State = State>,
    ExchangeKey: Debug + Clone + PartialEq,
    InstrumentKey: Debug + Clone + PartialEq,
{
    type Audit = ProcessEngineEventAudit;

    fn process(
        &mut self,
        event: EngineEvent<MarketEventKind, ExchangeKey, AssetKey, InstrumentKey>,
    ) -> Self::Audit {
        match &event {
            EngineEvent::Shutdown => {
                todo!()
            }
            EngineEvent::Command(command) => {
                let x = self.action(command);
            }
            EngineEvent::TradingStateUpdate(trading_state_update) => {
                let x = self.state.process(*trading_state_update);
            }
            EngineEvent::Account(account) => {
                let x = self.state.process(account);
            }
            EngineEvent::Market(market) => {
                self.state.process(market);
            }
        }

        ProcessEngineEventAudit::Shutdown
    }
}

pub enum ProcessEngineEventAudit {
    Shutdown,
    Action,
    TradingState,
    Account,
    Market,
}

impl<State, ExecutionTxs, StrategyT, Risk> Engine<State, ExecutionTxs, StrategyT, Risk> {
    pub fn new(
        clock: fn() -> DateTime<Utc>,
        state: State,
        execution_txs: ExecutionTxs,
        strategy: StrategyT,
        risk: Risk,
    ) -> Self {
        Self {
            sequence: 0,
            clock,
            state,
            execution_txs,
            strategy,
            risk,
        }
    }

    pub fn sequence_fetch_add(&mut self) -> u64 {
        let sequence = self.sequence;
        self.sequence += 1;
        sequence
    }
}

impl<AuditKind, State, ExecutionTx, StrategyT, Risk> Auditor<AuditKind>
    for Engine<State, ExecutionTx, StrategyT, Risk>
where
    AuditKind: From<State>,
    State: Clone,
{
    type Snapshot = State;

    fn snapshot(&self) -> Self::Snapshot {
        self.state.clone()
    }

    fn build_audit<Kind>(&mut self, kind: Kind) -> AuditEvent<AuditKind>
    where
        AuditKind: From<Kind>,
    {
        AuditEvent {
            id: self.sequence_fetch_add(),
            time: (self.clock)(),
            kind: AuditKind::from(kind),
        }
    }
}

// impl<ExecutionTxs, MarketState, StrategyT, Risk, ExchangeKey, AssetKey, InstrumentKey>
//     Processor<EngineEvent<MarketState::EventKind, ExchangeKey, AssetKey, InstrumentKey>>
//     for Engine<
//         EngineState<
//             MarketState,
//             StrategyT::State,
//             Risk::State,
//             ExchangeKey,
//             AssetKey,
//             InstrumentKey,
//         >,
//         ExecutionTxs,
//         StrategyT,
//         Risk,
//     >
// where
//     ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
//     MarketState: MarketDataState<InstrumentKey>,
//     StrategyT: Strategy<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
//     StrategyT::State: for<'a> Processor<&'a AccountEvent<ExchangeKey, AssetKey, InstrumentKey>>
//         + for<'a> Processor<&'a MarketEvent<InstrumentKey, MarketState::EventKind>>,
//     Risk: RiskManager<ExchangeKey, AssetKey>,
//     Risk::State: for<'a> Processor<&'a AccountEvent<ExchangeKey, AssetKey, InstrumentKey>>
//         + for<'a> Processor<&'a MarketEvent<InstrumentKey, MarketState::EventKind>>,
//     ExchangeKey: Debug + Clone,
//     AssetKey: Debug,
//     InstrumentKey: Debug + Clone,
//     EngineState<MarketState, StrategyT::State, Risk::State, ExchangeKey, AssetKey, InstrumentKey>:
//         StateManager<ExchangeId, State = ConnectivityState>
//             + StateManager<AssetKey, State = AssetState>
//             + StateManager<
//                 InstrumentKey,
//                 State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
//             >,
// {
//     type Audit = Audit<
//         EngineState<
//             MarketState,
//             StrategyT::State,
//             Risk::State,
//             ExchangeKey,
//             AssetKey,
//             InstrumentKey,
//         >,
//         EngineEvent<MarketState::EventKind, ExchangeKey, AssetKey, InstrumentKey>,
//         ExecutionRequestAudit<ExchangeKey, InstrumentKey>,
//     >;
//
//     fn process(
//         &mut self,
//         event: EngineEvent<MarketState::EventKind, ExchangeKey, AssetKey, InstrumentKey>,
//     ) -> Self::Audit {
//         match &event {
//             EngineEvent::Shutdown => return Audit::Shutdown(ShutdownAudit::Commanded(event)),
//             EngineEvent::Command(command) => {
//                 let output = self.action(command);
//
//                 return if let Some(unrecoverable) = output.unrecoverable_errors() {
//                     Audit::ShutdownWithOutput(ShutdownAudit::Error(event, unrecoverable), output)
//                 } else {
//                     Audit::ProcessWithOutput(event, output)
//                 };
//             }
//             EngineEvent::TradingStateUpdate(trading_state) => {
//                 self.state.update_from_trading_state_update(trading_state);
//             }
//             EngineEvent::Account(account) => {
//                 self.state.update_from_account(account);
//             }
//             EngineEvent::Market(market) => {
//                 self.state.update_from_market(market);
//             }
//         };
//
//         if let TradingState::Enabled = self.state.trading {
//             let output = self.trade();
//
//             if let Some(unrecoverable) = output.unrecoverable_errors() {
//                 Audit::ShutdownWithOutput(ShutdownAudit::Error(event, unrecoverable), output)
//             } else {
//                 Audit::ProcessWithOutput(event, output)
//             }
//         } else {
//             Audit::Process(event)
//         }
//     }
// }

pub type SendResult<ExchangeKey, InstrumentKey, Kind> = Result<
    Order<ExchangeKey, InstrumentKey, Kind>,
    (Order<ExchangeKey, InstrumentKey, Kind>, EngineError),
>;

impl<State, ExecutionTxs, Strategy, Risk> Engine<State, ExecutionTxs, Strategy, Risk> {
    pub fn action<ExchangeKey, InstrumentKey>(
        &mut self,
        command: &Command<ExchangeKey, InstrumentKey>,
    ) -> ActionOutput<ExchangeKey, InstrumentKey>
    where
        State: InstrumentStateManager<InstrumentKey, Exchange = ExchangeKey>,
        ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
        Strategy: CloseAllPositionsStrategy<ExchangeKey, InstrumentKey, State = State>,
        ExchangeKey: Debug + Clone + PartialEq,
        InstrumentKey: Debug + Clone + PartialEq,
    {
        match &command {
            Command::SendCancelRequests(requests) => {
                let output = self.send_requests(requests.clone());
                self.record_in_flight_cancels(&output.sent);
                ActionOutput::CancelOrders(output)
            }
            Command::SendOpenRequests(requests) => {
                let output = self.send_requests(requests.clone());
                self.record_in_flight_opens(&output.sent);
                ActionOutput::OpenOrders(output)
            }
            Command::ClosePositions(filter) => {
                ActionOutput::ClosePositions(self.close_positions(filter))
            }
            Command::CancelOrders(filter) => ActionOutput::CancelOrders(self.cancel_orders(filter)),
        }
    }

    pub fn send_requests<ExchangeKey, InstrumentKey, Kind>(
        &self,
        requests: impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, Kind>>,
    ) -> SendRequestsOutput<ExchangeKey, InstrumentKey, Kind>
    where
        ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
        ExchangeKey: Debug + Clone,
        InstrumentKey: Debug + Clone,
        Kind: Debug + Clone,
        ExecutionRequest<ExchangeKey, InstrumentKey>: From<Order<ExchangeKey, InstrumentKey, Kind>>,
    {
        // Send order requests
        let (sent, errors): (Vec<_>, Vec<_>) = requests
            .into_iter()
            .map(|request| {
                self.send_request(&request)
                    .map_err(|error| (request.clone(), error))
                    .map(|_| request)
            })
            .partition_result();

        SendRequestsOutput::new(NoneOneOrMany::from(sent), NoneOneOrMany::from(errors))
    }

    pub fn send_request<ExchangeKey, InstrumentKey, Kind>(
        &self,
        request: &Order<ExchangeKey, InstrumentKey, Kind>,
    ) -> Result<(), EngineError>
    where
        ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
        ExchangeKey: Debug + Clone,
        InstrumentKey: Debug + Clone,
        Kind: Debug + Clone,
        ExecutionRequest<ExchangeKey, InstrumentKey>: From<Order<ExchangeKey, InstrumentKey, Kind>>,
    {
        match self
            .execution_txs
            .find(&request.exchange)?
            .send(ExecutionRequest::from(request.clone()))
        {
            Ok(()) => Ok(()),
            Err(error) if error.is_unrecoverable() => {
                error!(
                    exchange = ?request.exchange,
                    ?request,
                    ?error,
                    "failed to send ExecutionRequest due to terminated channel"
                );
                Err(EngineError::Unrecoverable(
                    UnrecoverableEngineError::ExecutionChannelTerminated(format!(
                        "{:?} execution channel terminated: {:?}",
                        request.exchange, error
                    )),
                ))
            }
            Err(error) => {
                error!(
                    exchange = ?request.exchange,
                    ?request,
                    ?error,
                    "failed to send ExecutionRequest due to unhealthy channel"
                );
                Err(EngineError::Recoverable(
                    RecoverableEngineError::ExecutionChannelUnhealthy(format!(
                        "{:?} execution channel unhealthy: {:?}",
                        request.exchange, error
                    )),
                ))
            }
        }
    }

    pub fn record_in_flight_cancels<'a, ExchangeKey, InstrumentKey>(
        &mut self,
        requests: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestCancel>>,
    ) where
        State: InstrumentStateManager<InstrumentKey, Exchange = ExchangeKey>,
        ExchangeKey: Debug + Clone + 'a,
        InstrumentKey: Debug + Clone + 'a,
    {
        for request in requests.into_iter() {
            self.state
                .state_mut(&request.instrument)
                .orders
                .record_in_flight_cancel(request);
        }
    }

    pub fn record_in_flight_opens<'a, ExchangeKey, InstrumentKey>(
        &mut self,
        requests: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestOpen>>,
    ) where
        State: InstrumentStateManager<InstrumentKey, Exchange = ExchangeKey>,
        ExchangeKey: Debug + Clone + 'a,
        InstrumentKey: Debug + Clone + 'a,
    {
        for request in requests.into_iter() {
            self.state
                .state_mut(&request.instrument)
                .orders
                .record_in_flight_open(request);
        }
    }
}

// Todo: could further abstract State here...
impl<ExecutionTxs, MarketState, StrategyT, Risk, ExchangeKey, AssetKey, InstrumentKey>
    Engine<
        EngineState<
            MarketState,
            StrategyT::State,
            Risk::State,
            ExchangeKey,
            AssetKey,
            InstrumentKey,
        >,
        ExecutionTxs,
        StrategyT,
        Risk,
    >
where
    ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
    MarketState: MarketDataState<InstrumentKey>,
    StrategyT: Strategy<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    Risk: RiskManager<ExchangeKey, InstrumentKey>,
    ExchangeKey: Debug + Clone,
    InstrumentKey: Debug + Clone,
    EngineState<MarketState, StrategyT::State, Risk::State, ExchangeKey, AssetKey, InstrumentKey>:
        StateManager<AssetKey, State = AssetState>
            + StateManager<
                InstrumentKey,
                State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
            >,
{
    // pub fn action_execute_cancel(
    //     &mut self,
    //     request: &Order<ExchangeKey, InstrumentKey, RequestCancel>,
    // ) -> SentRequestsAudit<ExchangeKey, InstrumentKey> {
    //     todo!()
    //     // self.send_execution_request(request)
    //     //     .map(|_| SentRequestsAudit {
    //     //         cancels: NoneOneOrMany::One(request.clone()),
    //     //         ..Default::default()
    //     //     })
    //     //     .unwrap_or_else(|error| SentRequestsAudit {
    //     //         failed_cancels: NoneOneOrMany::One((request.clone(), EngineError::from(error))),
    //     //         ..Default::default()
    //     //     })
    // }
    //
    // pub fn send_execution_request<Kind>(
    //     &self,
    //     request: &Order<ExchangeKey, InstrumentKey, Kind>,
    // ) -> Result<(), UnrecoverableEngineError>
    // where
    //     Kind: Clone + Debug,
    //     ExecutionRequest<ExchangeKey, InstrumentKey>: From<Order<ExchangeKey, InstrumentKey, Kind>>,
    // {
    //     match self
    //         .execution_txs
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
    //
    // pub fn action_execute_open(
    //     &mut self,
    //     request: &Order<ExchangeKey, InstrumentKey, RequestOpen>,
    // ) -> SentRequestsAudit<ExchangeKey, InstrumentKey> {
    //     self.send_execution_request(request)
    //         .map(|_| SentRequestsAudit {
    //             opens: NoneOneOrMany::One(request.clone()),
    //             ..Default::default()
    //         })
    //         .unwrap_or_else(|error| SentRequestsAudit {
    //             failed_opens: NoneOneOrMany::One((request.clone(), EngineError::from(error))),
    //             ..Default::default()
    //         })
    // }
    //
    // pub fn close_position(
    //     &mut self,
    //     instrument: &InstrumentKey,
    // ) -> SentRequestsAudit<ExchangeKey, InstrumentKey> {
    //     // Generate orders
    //     let (cancels, opens) = self.strategy.close_position_request(
    //         instrument,
    //         &self.state.strategy,
    //         &self.state.assets,
    //         &self.state.instruments,
    //     );
    //
    //     // Bypass risk checks...
    //
    //     self.send_orders(cancels, opens)
    // }
    //
    // pub fn close_all_positions(&mut self) -> SentRequestsAudit<ExchangeKey, InstrumentKey> {
    //     // Generate orders
    //     let (cancels, opens) = self.strategy.close_all_positions_request(
    //         &self.state.strategy,
    //         &self.state.assets,
    //         &self.state.instruments,
    //     );
    //
    //     // Bypass risk checks...
    //
    //     self.send_orders(cancels, opens)
    // }
    //
    // pub fn cancel_order_by_id(
    //     &mut self,
    //     instrument: &InstrumentKey,
    //     id: &OrderId,
    // ) -> SentRequestsAudit<ExchangeKey, InstrumentKey> {
    //     // Todo: this evenings plan:
    //     //  1. Implement CancelAllOrders & CancelOrderById
    //     //  2. Re-design ExecutionManager to run request futures concurrently
    //
    //     // Todo: Open Q:
    //     // - Maybe CancelAllOrders, etc, should only be accessible via Command to keep audit flow
    //     //   in tact?
    //     // - Each method could have it's own custom Audit, with some parent enum
    //     //  eg/ CommandAudit::CancelAllOrders, etc. TradeAudit (mirror engine)
    //
    //     // For extendability, each piece of functionality could be a trait that the Engine
    //     // implements. eg/ trait CancelAllOrders, CancelOrderById, each with an Audit
    //
    //     todo!()
    //     // self.execution_tx.send(ExecutionRequest::CancelOrder(RequestCancel::new(instrument, id)))
    // }
    //
    // pub fn cancel_all_orders(&mut self) -> SentRequestsAudit<ExchangeKey, InstrumentKey> {
    //     // self.state.instruments
    //
    //     todo!()
    // }
    //
    // pub fn trade(&mut self) -> ExecutionRequestAudit<ExchangeKey, InstrumentKey> {
    //     // Generate orders
    //     let (cancels, opens) = self.strategy.generate_orders(
    //         &self.state.strategy,
    //         &self.state.assets,
    //         &self.state.instruments,
    //     );
    //
    //     // RiskApprove & RiskRefuse order requests
    //     let (cancels, opens, refused_cancels, refused_opens) = self.risk.check(
    //         &self.state.risk,
    //         &self.state.assets,
    //         &self.state.instruments,
    //         cancels,
    //         opens,
    //     );
    //
    //     // Send risk approved order requests
    //     let sent = self.send_orders(
    //         cancels.into_iter().map(|RiskApproved(cancel)| cancel),
    //         opens.into_iter().map(|RiskApproved(open)| open),
    //     );
    //
    //     // Collect remaining Iterators (so we can &mut self)
    //     let refused = RiskRefusedRequestsAudit {
    //         refused_cancels: refused_cancels.into_iter().collect(),
    //         refused_opens: refused_opens.into_iter().collect(),
    //     };
    //
    //     // Record in flight order requests
    //     self.record_requests_in_flight(&sent.cancels, &sent.opens);
    //
    //     ExecutionRequestAudit { sent, refused }
    // }
    //
    // pub fn send_orders(
    //     &self,
    //     cancels: impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestCancel>>,
    //     opens: impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestOpen>>,
    // ) -> SentRequestsAudit<ExchangeKey, InstrumentKey> {
    //     // Send order requests
    //     let (cancels_sent, cancel_send_errs): (Vec<_>, Vec<_>) = cancels
    //         .into_iter()
    //         .map(|cancel| {
    //             self.send_execution_request(&cancel)
    //                 .map_err(|error| (cancel.clone(), EngineError::from(error)))
    //                 .map(|_| cancel)
    //         })
    //         .partition_result();
    //
    //     let (opens_sent, open_send_errs): (Vec<_>, Vec<_>) = opens
    //         .into_iter()
    //         .map(|open| {
    //             self.send_execution_request(&open)
    //                 .map_err(|error| (open.clone(), EngineError::from(error)))
    //                 .map(|_| open)
    //         })
    //         .partition_result();
    //
    //     SentRequestsAudit {
    //         cancels: NoneOneOrMany::Many(cancels_sent),
    //         opens: NoneOneOrMany::Many(opens_sent),
    //         failed_cancels: NoneOneOrMany::from(cancel_send_errs),
    //         failed_opens: NoneOneOrMany::from(open_send_errs),
    //     }
    // }

    // pub fn record_requests_in_flight<'a>(
    //     &mut self,
    //     cancels: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestCancel>>,
    //     opens: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestOpen>>,
    // ) where
    //     ExchangeKey: 'a,
    //     InstrumentKey: 'a,
    // {
    //     for request in cancels.into_iter() {
    //         self.record_request_cancel_in_flight(request);
    //     }
    //     for request in opens.into_iter() {
    //         self.record_request_open_in_flight(request);
    //     }
    // }
    //
    // pub fn record_in_flight_cancels(
    //     &mut self,
    //     requests: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestCancel>>,
    // ) {
    //     for request in requests.into_iter() {
    //         self.state
    //             .state_mut(&request.instrument)
    //             .orders
    //             .record_in_flight_cancel(request);
    //     }
    // }
    //
    // pub fn record_in_flight_opens(
    //     &mut self,
    //     requests: impl IntoIterator<Item = &Order<ExchangeKey, InstrumentKey, RequestOpen>>,
    // )
    // where
    //     Self: StateManager<InstrumentKey, State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>>
    // {
    //     for request in requests.into_iter() {
    //         self.state
    //             .state_mut(&request.instrument)
    //             .orders
    //             .record_in_flight_open(request);
    //     }
}

pub fn record_in_flight_cancels<'a, State, MarketState, ExchangeKey, AssetKey, InstrumentKey>(
    state: &'a mut State,
    requests: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestCancel>>,
) where
    State: StateManager<
        InstrumentKey,
        State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    >,
    ExchangeKey: Debug + Clone + 'a,
    InstrumentKey: Debug + Clone + 'a,
{
    for request in requests.into_iter() {
        state
            .state_mut(&request.instrument)
            .orders
            .record_in_flight_cancel(request);
    }
}

pub fn record_in_flight_opens<'a, State, MarketState, ExchangeKey, AssetKey, InstrumentKey>(
    state: &'a mut State,
    requests: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestOpen>>,
) where
    State: StateManager<
        InstrumentKey,
        State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    >,
    ExchangeKey: Debug + Clone + 'a,
    InstrumentKey: Debug + Clone + 'a,
{
    for request in requests.into_iter() {
        state
            .state_mut(&request.instrument)
            .orders
            .record_in_flight_open(request);
    }
}

pub trait InstrumentStateManager<InstrumentKey> {
    type MarketState: Debug + Clone;
    type Exchange: Debug + Clone;
    type Asset: Debug + Clone;

    fn state(
        &self,
        key: &InstrumentKey,
    ) -> &InstrumentState<Self::MarketState, Self::Exchange, Self::Asset, InstrumentKey>;

    fn state_mut(
        &mut self,
        key: &InstrumentKey,
    ) -> &mut InstrumentState<Self::MarketState, Self::Exchange, Self::Asset, InstrumentKey>;

    fn states_by_filter<'a>(
        &'a self,
        filter: &InstrumentFilter<Self::Exchange, InstrumentKey>,
    ) -> impl Iterator<
        Item = &'a InstrumentState<Self::MarketState, Self::Exchange, Self::Asset, InstrumentKey>,
    >
    where
        Self::MarketState: 'a,
        Self::Exchange: 'a,
        Self::Asset: 'a,
        InstrumentKey: 'a;
}

impl<MarketState, Strategy, Risk> InstrumentStateManager<InstrumentIndex>
    for IndexedEngineState<MarketState, Strategy, Risk>
where
    MarketState: Debug + Clone,
{
    type MarketState = MarketState;
    type Exchange = ExchangeIndex;
    type Asset = AssetIndex;

    fn state(
        &self,
        key: &InstrumentIndex,
    ) -> &InstrumentState<MarketState, ExchangeIndex, AssetIndex, InstrumentIndex> {
        todo!()
    }

    fn state_mut(
        &mut self,
        key: &InstrumentIndex,
    ) -> &mut InstrumentState<MarketState, ExchangeIndex, AssetIndex, InstrumentIndex> {
        todo!()
    }

    fn states_by_filter<'a>(
        &'a self,
        filter: &InstrumentFilter<ExchangeIndex, InstrumentIndex>,
    ) -> impl Iterator<Item = &'a InstrumentState<MarketState, ExchangeIndex, AssetIndex, InstrumentIndex>>
    where
        MarketState: 'a,
    {
        std::iter::empty()
    }
}

impl<MarketState, Strategy, Risk> InstrumentStateManager<InstrumentNameInternal>
    for EngineState<
        MarketState,
        Strategy,
        Risk,
        ExchangeId,
        AssetNameInternal,
        InstrumentNameInternal,
    >
where
    MarketState: Debug + Clone,
{
    type MarketState = MarketState;
    type Exchange = ExchangeId;
    type Asset = AssetNameInternal;

    fn state(
        &self,
        key: &InstrumentNameInternal,
    ) -> &InstrumentState<MarketState, ExchangeId, AssetNameInternal, InstrumentNameInternal> {
        todo!()
    }

    fn state_mut(
        &mut self,
        key: &InstrumentNameInternal,
    ) -> &mut InstrumentState<MarketState, ExchangeId, AssetNameInternal, InstrumentNameInternal>
    {
        todo!()
    }

    fn states_by_filter<'a>(
        &'a self,
        filter: &InstrumentFilter<ExchangeId, InstrumentNameInternal>,
    ) -> impl Iterator<
        Item = &'a InstrumentState<
            MarketState,
            ExchangeId,
            AssetNameInternal,
            InstrumentNameInternal,
        >,
    >
    where
        MarketState: 'a,
    {
        std::iter::empty()
    }
}
