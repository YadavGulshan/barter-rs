use crate::v2::{
    engine::{
        action::send_requests::SendRequestsOutput, execution_tx::ExecutionTxMap, Engine,
        InstrumentStateManager,
    },
    order::{Order, RequestCancel, RequestOpen},
    risk::{RiskApproved, RiskManager, RiskRefused},
};
use barter_integration::collection::none_one_or_many::NoneOneOrMany;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub trait GenerateAlgoOrders<ExchangeKey, InstrumentKey> {
    type Output;
    fn generate_algo_orders(&mut self) -> Self::Output;
}

pub trait AlgoStrategy<ExchangeKey, InstrumentKey> {
    type State;

    fn generate_orders(
        &self,
        state: &Self::State,
    ) -> (
        impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestCancel>>,
        impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestOpen>>,
    );
}

impl<State, ExecutionTxs, Strategy, Risk, ExchangeKey, InstrumentKey>
    GenerateAlgoOrders<ExchangeKey, InstrumentKey> for Engine<State, ExecutionTxs, Strategy, Risk>
where
    State: InstrumentStateManager<InstrumentKey, Exchange = ExchangeKey>,
    ExecutionTxs: ExecutionTxMap<ExchangeKey, InstrumentKey>,
    Strategy: AlgoStrategy<ExchangeKey, InstrumentKey, State = State>,
    Risk: RiskManager<ExchangeKey, InstrumentKey, State = State>,
    ExchangeKey: Debug + Clone,
    InstrumentKey: Debug + Clone,
{
    type Output = GenerateAlgoOrdersOutput<ExchangeKey, InstrumentKey>;

    fn generate_algo_orders(&mut self) -> Self::Output {
        // Generate orders
        let (cancels, opens) = self.strategy.generate_orders(&self.state);

        // RiskApprove & RiskRefuse order requests
        let (cancels, opens, refused_cancels, refused_opens) =
            self.risk.check(&self.state, cancels, opens);

        // Send risk approved order requests
        let cancels = self.send_requests(cancels.into_iter().map(|RiskApproved(cancel)| cancel));
        let opens = self.send_requests(opens.into_iter().map(|RiskApproved(open)| open));

        // Collect remaining Iterators (so we can access &mut self)
        let cancels_refused = refused_cancels.into_iter().collect();
        let opens_refused = refused_opens.into_iter().collect();

        // Record in flight order requests
        self.record_in_flight_cancels(cancels.sent.iter());
        self.record_in_flight_opens(opens.sent.iter());

        GenerateAlgoOrdersOutput::new(cancels, cancels_refused, opens, opens_refused)
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Constructor)]
pub struct GenerateAlgoOrdersOutput<ExchangeKey, InstrumentKey> {
    pub cancels: SendRequestsOutput<ExchangeKey, InstrumentKey, RequestCancel>,
    pub cancels_refused:
        NoneOneOrMany<RiskRefused<Order<ExchangeKey, InstrumentKey, RequestCancel>>>,
    pub opens: SendRequestsOutput<ExchangeKey, InstrumentKey, RequestOpen>,
    pub opens_refused: NoneOneOrMany<RiskRefused<Order<ExchangeKey, InstrumentKey, RequestOpen>>>,
}
