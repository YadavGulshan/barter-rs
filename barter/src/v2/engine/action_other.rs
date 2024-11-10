// pub trait EngineAction<Kind> {
//     type State<'a>;
//
//     type Params;
//     type Audit: &mut Self;
// }

// pub trait SendOrders<ExchangeKey, InstrumentKey>: EngineAction  {
//     fn send_orders(
//         &mut self,
//         cancels: impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestCancel>>,
//         opens: impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestOpen>>,
//     ) -> Self::Audit;
// }
//
// pub trait StateAction<Kind> {
//
// }

// pub trait EngineAction {
//     type Engine;
//     type Params;
//     type Audit;
// }
//
// pub struct CancelOrderByIdAction;
//
// impl EngineAction for CancelOrderByIdAction {
//     type Engine = ();
//     type Params = ();
//     type Audit = ();
// }
