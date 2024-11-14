use crate::v2::{
    engine::{
        action::{
            cancel_orders::CancelOrders,
            close_positions::{ClosePositions, ClosePositionsStrategy},
            send_requests::SendRequestsOutput,
            ActionOutput,
        },
        audit::{AuditEvent, Auditor},
        command::{Command, InstrumentFilter},
        error::{EngineError, RecoverableEngineError, UnrecoverableEngineError},
        execution_tx::ExecutionTxMap,
        state::{
            instrument::{market_data::MarketDataState, InstrumentState},
            order_manager::OrderManager,
            trading_state_manager::TradingStateManager,
            EngineState, IndexedEngineState,
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
    State: TradingStateManager
        + for<'a> Processor<&'a MarketStreamEvent<InstrumentKey, MarketEventKind>>
        + for<'a> Processor<&'a AccountStreamEvent<ExchangeKey, AssetKey, InstrumentKey>>,
    ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
    Strategy: ClosePositionsStrategy<ExchangeKey, InstrumentKey, State = State>,
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
