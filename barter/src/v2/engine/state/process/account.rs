use crate::v2::{
    engine::{
        state::{
            order_manager::OrderManager, AssetStateManager,
            ConnectivityManager, EngineState, InstrumentStateManager,
        },
        Processor,
    },
    execution::{AccountEvent, AccountEventKind},
    Snapshot,
};
use barter_instrument::exchange::ExchangeId;
use std::fmt::Debug;

impl<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
    Processor<&AccountEvent<ExchangeKey, AssetKey, InstrumentKey>>
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey>
where
    Self: ConnectivityManager<ExchangeId>
        + AssetStateManager<AssetKey>
        + InstrumentStateManager<InstrumentKey, ExchangeKey = ExchangeKey, AssetKey = AssetKey>,
    Strategy: for<'a> Processor<&'a AccountEvent<ExchangeKey, AssetKey, InstrumentKey>>,
    Risk: for<'a> Processor<&'a AccountEvent<ExchangeKey, AssetKey, InstrumentKey>>,
    ExchangeKey: Debug + Clone,
    AssetKey: Debug,
    InstrumentKey: Debug + Clone,
{
    type Audit = ProcessAccountEventAudit;

    fn process(
        &mut self,
        event: &AccountEvent<ExchangeKey, AssetKey, InstrumentKey>,
    ) -> Self::Audit {
        // Todo: set exchange ConnectivityState to healthy if unhealthy

        match &event.kind {
            AccountEventKind::Snapshot(snapshot) => {
                for balance in &snapshot.balances {
                    self.asset_mut(&balance.asset)
                        .update_from_balance(Snapshot(balance))
                }
                for instrument in &snapshot.instruments {
                    self.instrument_mut(&instrument.position.instrument)
                        .update_from_account_snapshot(instrument)
                }
            }
            AccountEventKind::BalanceSnapshot(balance) => {
                self.asset_mut(&balance.0.asset)
                    .update_from_balance(balance.as_ref());
            }
            AccountEventKind::PositionSnapshot(position) => {
                self.instrument_mut(&position.0.instrument)
                    .update_from_position_snapshot(position.as_ref());
            }
            AccountEventKind::OrderSnapshot(order) => self
                .instrument_mut(&order.0.instrument)
                .orders
                .update_from_order_snapshot(order.as_ref()),
            AccountEventKind::OrderOpened(response) => self
                .instrument_mut(&response.instrument)
                .orders
                .update_from_open(response),
            AccountEventKind::OrderCancelled(response) => self
                .instrument_mut(&response.instrument)
                .orders
                .update_from_cancel(response),
            AccountEventKind::Trade(trade) => {
                self.instrument_mut(&trade.instrument)
                    .update_from_trade(trade);
            }
        }

        // Update any user provided Strategy & Risk State
        self.strategy.process(event);
        self.risk.process(event);

        ProcessAccountEventAudit
    }
}

pub struct ProcessAccountEventAudit;
