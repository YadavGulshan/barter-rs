pub mod action;

// Decoupled from Auditor and ExecutionTxs
pub struct Engine<State, StrategyT, Risk, ExecutionTxs> {
    // pub sequence: u64,
    // pub clock: fn() -> DateTime<Utc>,
    pub state: State,
    pub strategy: StrategyT,
    pub risk: Risk,
    pub execution_txs: ExecutionTxs,
}

// impl<ExecutionTxs, MarketState, StrategyT, Risk, ExchangeKey, AssetKey, InstrumentKey>
// Processor<EngineEvent<MarketState::EventKind, ExchangeKey, AssetKey, InstrumentKey>>
// for Engine<
//     EngineState<
//         MarketState,
//         StrategyT::State,
//         Risk::State,
//         ExchangeKey,
//         AssetKey,
//         InstrumentKey,
//     >,
//     ExecutionTxs,
//     StrategyT,
//     Risk,
// >
// where
//     ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
//     MarketState: MarketDataState<InstrumentKey>,
//     StrategyT: Strategy<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
//     StrategyT::State: for<'a> Processor<&'a AccountEvent<ExchangeKey, AssetKey, InstrumentKey>>
//     + for<'a> Processor<&'a MarketEvent<InstrumentKey, MarketState::EventKind>>,
//     Risk: RiskManager<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
//     Risk::State: for<'a> Processor<&'a AccountEvent<ExchangeKey, AssetKey, InstrumentKey>>
//     + for<'a> Processor<&'a MarketEvent<InstrumentKey, MarketState::EventKind>>,
//     ExchangeKey: Debug + Clone,
//     AssetKey: Debug,
//     InstrumentKey: Debug + Clone,
//     EngineState<MarketState, StrategyT::State, Risk::State, ExchangeKey, AssetKey, InstrumentKey>:
//     StateManager<ExchangeId, State = ConnectivityState>
//     + StateManager<AssetKey, State = AssetState>
//     + StateManager<
//         InstrumentKey,
//         State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
//     >,
// {
//     type Output = ();
//
//     fn process(
//         &mut self,
//         event: EngineEvent<MarketState::EventKind, ExchangeKey, AssetKey, InstrumentKey>,
//     ) -> Self::Output {
//         match &event {
//             EngineEvent::Shutdown => {}
//             EngineEvent::Command(_) => {}
//             EngineEvent::TradingStateUpdate(_) => {}
//             EngineEvent::Account(_) => {}
//             EngineEvent::Market(_) => {}
//         }
//
//
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
