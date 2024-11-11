use crate::v2::{
    engine::state::{
        instrument::InstrumentState, order_manager::OrderManager, EngineState, StateManager,
    },
    order::{Order, RequestCancel, RequestOpen},
};
use std::fmt::Debug;

pub trait OrdersInFlightRecorder<ExchangeKey, InstrumentKey> {
    type Audit;

    fn record_orders_in_flight<'a>(
        &mut self,
        cancels: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestCancel>>,
        opens: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestOpen>>,
    ) -> Self::Audit
    where
        ExchangeKey: 'a,
        InstrumentKey: 'a;
}

pub struct OrdersInFlightRecorderAudit;

impl<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
    OrdersInFlightRecorder<ExchangeKey, InstrumentKey>
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
where
    Self: StateManager<
        InstrumentKey,
        State = InstrumentState<Market, ExchangeKey, AssetKey, InstrumentKey>,
    >,
    ExchangeKey: Debug + Clone,
    InstrumentKey: Debug + Clone,
{
    type Audit = OrdersInFlightRecorderAudit;

    fn record_orders_in_flight<'a>(
        &mut self,
        cancels: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestCancel>>,
        opens: impl IntoIterator<Item = &'a Order<ExchangeKey, InstrumentKey, RequestOpen>>,
    ) -> Self::Audit
    where
        ExchangeKey: 'a,
        InstrumentKey: 'a,
    {
        for request in cancels.into_iter() {
            self.state_mut(&request.instrument)
                .orders
                .record_in_flight_cancel(request);
        }
        for request in opens.into_iter() {
            self.state_mut(&request.instrument)
                .orders
                .record_in_flight_open(request);
        }

        OrdersInFlightRecorderAudit
    }
}
