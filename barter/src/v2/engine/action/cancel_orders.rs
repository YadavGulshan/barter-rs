use crate::v2::{
    engine::{
        action::send_requests::SendRequestsOutput,
        command::InstrumentFilter,
        execution_tx::ExecutionTxMap,
        state::order_manager::{InFlightRequestRecorder, OrderManager},
        Engine,
    },
    order::{Order, RequestCancel},
};
use std::fmt::Debug;

pub trait CancelOrders<ExchangeKey, InstrumentKey> {
    type Output;

    fn cancel_orders(
        &mut self,
        filter: &InstrumentFilter<ExchangeKey, InstrumentKey>,
    ) -> Self::Output;
}

impl<State, ExecutionTxs, Strategy, Risk, ExchangeKey, InstrumentKey>
    CancelOrders<ExchangeKey, InstrumentKey> for Engine<State, ExecutionTxs, Strategy, Risk>
where
    State: InFlightRequestRecorder<ExchangeKey, InstrumentKey>,
    ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
    ExchangeKey: Debug + Clone + PartialEq,
    InstrumentKey: Debug + Clone + PartialEq,
{
    type Output = SendRequestsOutput<ExchangeKey, InstrumentKey, RequestCancel>;

    fn cancel_orders(
        &mut self,
        filter: &InstrumentFilter<ExchangeKey, InstrumentKey>,
    ) -> Self::Output {
        let requests = self
            .state
            .states_by_filter(filter)
            .flat_map(|state| state.orders.orders().filter_map(Order::as_request_cancel));

        // Bypass risk checks...

        // Send order requests
        let cancels = self.send_requests(requests);

        // Record in flight order requests
        self.state.record_in_flight_cancels(&cancels.sent);

        cancels
    }
}
