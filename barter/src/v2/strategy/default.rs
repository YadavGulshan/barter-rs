// #[derive(Debug, Clone)]
// pub struct DefaultStrategy;
//
// impl<MarketState, ExchangeKey, AssetKey, InstrumentKey>
//     Strategy<MarketState, ExchangeKey, AssetKey, InstrumentKey> for DefaultStrategy
// {
//     type State = DefaultStrategyState;
//
//     fn generate_orders(
//         &self,
//         _: &Self::State,
//         _: &AssetStates,
//         _: &InstrumentStates<MarketState, ExchangeKey, AssetKey, InstrumentKey>,
//     ) -> (
//         impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestCancel>>,
//         impl IntoIterator<Item = Order<ExchangeKey, InstrumentKey, RequestOpen>>,
//     ) {
//         (std::iter::empty(), std::iter::empty())
//     }
// }
//
// #[derive(Debug, Clone)]
// pub struct DefaultStrategyState;
//
// impl<ExchangeKey, AssetKey, InstrumentKey>
//     Processor<&AccountEvent<ExchangeKey, AssetKey, InstrumentKey>> for DefaultStrategyState
// {
//     type Audit = ();
//     fn process(&mut self, _: &AccountEvent<ExchangeKey, AssetKey, InstrumentKey>) -> Self::Audit {}
// }
//
// impl<InstrumentKey, Kind> Processor<&MarketEvent<InstrumentKey, Kind>> for DefaultStrategyState {
//     type Audit = ();
//     fn process(&mut self, _: &MarketEvent<InstrumentKey, Kind>) -> Self::Audit {}
// }
