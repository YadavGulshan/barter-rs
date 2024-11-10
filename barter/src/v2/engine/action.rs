use crate::v2::{
    engine::{
        state::{
            asset::AssetState, instrument::InstrumentState, order_manager::OrderManager,
            EngineState, StateManager,
        },
        Engine,
    },
    order::{Order, RequestCancel, RequestOpen},
    risk::RiskManager,
    strategy::Strategy,
};
use std::fmt::Debug;

pub trait CloseAllPositions<ExchangeKey, InstrumentKey> {
    type Audit;

    fn close_all_positions(
        &mut self,
        // exchange: Option<ExchangeKey>
    ) -> Self::Audit;
}

pub trait CloseAllPositionsStrategy<ExchangeKey, InstrumentKey> {
    type State;

    fn close_all_positions_request(
        &self,
        state: Self::State,
    ) -> (
        impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestCancel>>,
        impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestOpen>>,
    );
}

// pub struct AuditCloseAllPositions;
//
// impl<ExecutionTxs, Strategy, Risk, ExchangeKey, InstrumentKey> CloseAllPositions<ExchangeKey, InstrumentKey>
// for Engine<Strategy::State, ExecutionTxs, Strategy, Risk>
// where
//     Strategy: CloseAllPositionsStrategy<ExchangeKey, InstrumentKey>,
// {
//     type Audit = AuditCloseAllPositions;
//
//     fn close_all_positions(&self) -> Self::Audit {
//         // Generate orders
//         let (
//             cancels,
//             opens
//         ) = self.strategy.close_all_positions_request(&self.state);
//
//
//     }
// }

// Todo: de-couple Engine from tx
// pub trait SendOrders<ExchangeKey, InstrumentKey> {
//     type Audit;
//
//     fn send_orders(
//         &mut self,
//         tx: impl ExecutionTxMap<ExchangeKey, InstrumentKey>, // Now we are more granular, better to decouple
//         cancels: impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestCancel>>,
//         opens: impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestOpen>>,
//     ) -> Self::Audit;
// }

// impl<ExchangeKey, InstrumentKey> SendOrders<ExchangeKey, InstrumentKey>
// for Engine<>

pub trait RecordOrdersInFlight<ExchangeKey, InstrumentKey> {
    type Audit;

    fn record_requests_in_flight<'a>(
        &mut self,
        cancels: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestCancel>>,
        opens: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestOpen>>,
    ) -> Self::Audit
    where
        ExchangeKey: 'a,
        InstrumentKey: 'a;
}

pub struct AuditRecordOrdersInFlight;

impl<MarketState, ExecutionTxs, StrategyT, Risk, ExchangeKey, AssetKey, InstrumentKey>
    RecordOrdersInFlight<ExchangeKey, InstrumentKey>
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
    StrategyT: Strategy<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    Risk: RiskManager<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
    EngineState<MarketState, StrategyT::State, Risk::State, ExchangeKey, AssetKey, InstrumentKey>:
        StateManager<AssetKey, State = AssetState>
            + StateManager<
                InstrumentKey,
                State = InstrumentState<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
            >,
    ExchangeKey: Debug + Clone,
    InstrumentKey: Debug + Clone,
{
    type Audit = AuditRecordOrdersInFlight;

    fn record_requests_in_flight<'a>(
        &mut self,
        cancels: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestCancel>>,
        opens: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestOpen>>,
    ) -> Self::Audit
    where
        ExchangeKey: 'a,
        InstrumentKey: 'a,
    {
        for request in cancels.into_iter() {
            self.state
                .state_mut(&request.instrument)
                .orders
                .record_in_flight_cancel(request);
        }
        for request in opens.into_iter() {
            self.state
                .state_mut(&request.instrument)
                .orders
                .record_in_flight_open(request);
        }

        AuditRecordOrdersInFlight
    }
}
