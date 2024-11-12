use crate::v2::execution::AccountEvent;

pub trait AccountEventUpdater<ExchangeKey, AssetKey, InstrumentKey> {
    type Audit; 
    fn update_from_account(
        &mut self,
        event: &AccountEvent<ExchangeKey, AssetKey, InstrumentKey>
    )
}