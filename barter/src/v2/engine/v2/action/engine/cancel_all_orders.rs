use crate::v2::{
    engine::{
        execution_tx::ExecutionTxMap,
        state::{
            instrument::InstrumentState, order_manager::OrderManager, EngineState, StateManager,
        },
        v2::{
            action::engine::{send_requests, SendRequestsOutput},
            Engine,
        },
    },
    order::{Order, RequestCancel},
    risk::RiskManager,
    strategy::Strategy,
};
use itertools::Either;
use std::fmt::Debug;

pub trait CancelAllOrders<ExchangeKey, InstrumentKey> {
    type Output;

    fn cancel_all_orders(
        &mut self,
        exchange: Option<&ExchangeKey>,
        txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>,
    ) -> Self::Output;
}

impl<MarketState, StrategyT, Risk, ExchangeKey, InstrumentKey, AssetKey>
    CancelAllOrders<ExchangeKey, InstrumentKey>
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
    StrategyT: Strategy<MarketState, AssetKey, ExchangeKey, InstrumentKey>,
    Risk: RiskManager<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    ExchangeKey: Debug + Clone + PartialEq,
    InstrumentKey: Debug + Clone,
    EngineState<MarketState, StrategyT::State, Risk::State, ExchangeKey, AssetKey, InstrumentKey>:
        StateManager<
            InstrumentKey,
            State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
        >,
{
    type Output = SendRequestsOutput<ExchangeKey, InstrumentKey, RequestCancel>;

    fn cancel_all_orders(
        &mut self,
        exchange: Option<&ExchangeKey>,
        txs: &impl ExecutionTxMap<ExchangeKey, InstrumentKey>,
    ) -> Self::Output {
        let requests = match exchange {
            // Cancel all orders on an exchange
            Some(exchange) => Either::Left(
                self.state
                    .instruments
                    .states_by_exchange(exchange)
                    .flat_map(|state| state.orders.orders().filter_map(Order::as_request_cancel)),
            ),
            // Cancel all orders across all exchanges
            None => Either::Right(
                self.state
                    .instruments
                    .states()
                    .flat_map(|state| state.orders.orders().filter_map(Order::as_request_cancel)),
            ),
        };

        send_requests(txs, requests)
    }
}
