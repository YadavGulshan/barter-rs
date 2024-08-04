use barter_instrument::Side;
use derive_more::Constructor;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize, Constructor)]
pub struct Position<InstrumentKey> {
    pub instrument: InstrumentKey,
    pub side: Side,
    pub quantity_net: Decimal,
    pub price_average: Decimal,
    pub pnl_unrealised: Decimal,
    pub pnl_realised: Decimal,
}

impl<InstrumentKey> Position<InstrumentKey> {
    pub fn new_flat(instrument: InstrumentKey) -> Self {
        Self {
            instrument,
            side: Side::Buy,
            quantity_net: Decimal::ZERO,
            price_average: Decimal::ZERO,
            pnl_unrealised: Decimal::ZERO,
            pnl_realised: Decimal::ZERO,
        }
    }
}
