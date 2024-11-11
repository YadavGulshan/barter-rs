use crate::v2::{
    engine::{
        action::send_requests::{send_requests, SendRequestsOutput},
        execution_tx::ExecutionTxMap,
        state::{instrument::InstrumentState, EngineState, StateManager},
        Engine,
    },
    order::{Order, RequestCancel, RequestOpen},
    risk::{RiskApproved, RiskManager, RiskRefused},
    strategy::Strategy,
};
use barter_integration::collection::none_one_or_many::NoneOneOrMany;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
// Todo: - AlgoStrategy would maybe live in here, like CloseAllPositions, etc.

pub trait GenerateAlgoOrders<ExchangeKey, InstrumentKey> {
    type Output;
    fn generate_algo_orders(
        &mut self,
        txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>,
    ) -> Self::Output;
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Constructor)]
pub struct GenerateAlgoOrdersOutput<ExchangeKey, InstrumentKey> {
    pub cancels: SendRequestsOutput<ExchangeKey, InstrumentKey, RequestCancel>,
    pub cancels_refused:
        NoneOneOrMany<RiskRefused<Order<ExchangeKey, InstrumentKey, RequestCancel>>>,
    pub opens: SendRequestsOutput<ExchangeKey, InstrumentKey, RequestOpen>,
    pub opens_refused: NoneOneOrMany<RiskRefused<Order<ExchangeKey, InstrumentKey, RequestOpen>>>,
}

impl<MarketState, ExecutionTxs, StrategyT, Risk, ExchangeKey, AssetKey, InstrumentKey>
    GenerateAlgoOrders<ExchangeKey, InstrumentKey>
    for Engine<
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
    StrategyT: Strategy<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    Risk: RiskManager<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    ExchangeKey: Debug + Clone,
    InstrumentKey: Debug + Clone,
    EngineState<MarketState, StrategyT::State, Risk::State, ExchangeKey, AssetKey, InstrumentKey>:
        StateManager<
            InstrumentKey,
            State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
        >,
{
    type Output = GenerateAlgoOrdersOutput<ExchangeKey, InstrumentKey>;

    fn generate_algo_orders(
        &mut self,
        txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>,
    ) -> Self::Output {
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
        let cancels = send_requests(txs, cancels.into_iter().map(|RiskApproved(cancel)| cancel));
        let opens = send_requests(txs, opens.into_iter().map(|RiskApproved(open)| open));

        // Collect remaining Iterators (so we can access &mut self)
        let cancels_refused = refused_cancels.into_iter().collect();
        let opens_refused = refused_opens.into_iter().collect();

        // Record in flight order requests
        self.state.record_in_flight_cancels(cancels.sent.iter());
        self.state.record_in_flight_opens(opens.sent.iter());

        GenerateAlgoOrdersOutput::new(cancels, cancels_refused, opens, opens_refused)
    }
}
