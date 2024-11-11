use crate::v2::{
    engine::{
        command::InstrumentFilter,
        execution_tx::ExecutionTxMap,
        state::{
            instrument::InstrumentState, order_manager::OrderManager, EngineState, StateManager,
        },
        v2::{
            action::engine::send_requests::{send_requests, SendRequestsOutput},
            Engine,
        },
    },
    order::{Order, RequestCancel},
    risk::RiskManager,
    strategy::Strategy,
};
use std::fmt::Debug;

pub trait CancelOrders<ExchangeKey, InstrumentKey> {
    type Output;

    fn cancel_orders(
        &mut self,
        filter: &InstrumentFilter<ExchangeKey, InstrumentKey>,
    ) -> Self::Output;
}

impl<MarketState, StrategyT, Risk, ExecutionTxs, ExchangeKey, InstrumentKey, AssetKey>
    CancelOrders<ExchangeKey, InstrumentKey>
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
        ExecutionTxs,
    >
where
    StrategyT: Strategy<MarketState, AssetKey, ExchangeKey, InstrumentKey>,
    Risk: RiskManager<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
    ExchangeKey: Debug + Clone + PartialEq,
    InstrumentKey: Debug + Clone + PartialEq,
    EngineState<MarketState, StrategyT::State, Risk::State, ExchangeKey, AssetKey, InstrumentKey>:
        StateManager<
            InstrumentKey,
            State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
        >,
{
    type Output = SendRequestsOutput<ExchangeKey, InstrumentKey, RequestCancel>;

    fn cancel_orders(
        &mut self,
        filter: &InstrumentFilter<ExchangeKey, InstrumentKey>,
    ) -> Self::Output {
        let requests = self
            .state
            .instruments
            .states_by_filter(filter)
            .flat_map(|state| state.orders.orders().filter_map(Order::as_request_cancel));

        send_requests(&self.execution_txs, requests)
    }
}
