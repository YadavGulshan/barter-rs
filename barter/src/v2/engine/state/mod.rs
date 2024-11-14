use crate::v2::engine::{
    command::InstrumentFilter,
    state::{
        asset::{AssetState, AssetStates},
        connectivity::{ConnectivityState, ConnectivityStates},
        instrument::{InstrumentState, InstrumentStates},
        trading_state_manager::TradingState,
    },
    Processor,
};
use barter_instrument::{
    asset::{name::AssetNameInternal, AssetIndex, ExchangeAsset},
    exchange::{ExchangeId, ExchangeIndex},
    instrument::{name::InstrumentNameInternal, InstrumentIndex},
};
use itertools::Either;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod asset;
pub mod connectivity;
pub mod instrument;
pub mod order_manager;
pub mod process;
pub mod trading_state_manager;

// Todo:
//  - Maybe introduce State machine for dealing with connectivity VecMap issue...
//    '--> could only check if a new Account/Market event updates to Connected if we are in
//         State=Unhealthy, that way we are only doing expensive lookup in that case
//  - Need to make some Key decisions about "what is a manager", and "what is an Updater"

pub type IndexedEngineState<Market, Strategy, Risk> =
    EngineState<Market, Strategy, Risk, ExchangeIndex, AssetIndex, InstrumentIndex>;

pub trait ConnectivityManager<ExchangeKey> {
    fn connectivity(&self, key: &ExchangeKey) -> &ConnectivityState;
    fn connectivity_mut(&mut self, key: &ExchangeKey) -> &mut ConnectivityState;
}

pub trait AssetStateManager<AssetKey> {
    fn asset(&self, key: &AssetKey) -> &AssetState;
    fn asset_mut(&mut self, key: &AssetKey) -> &mut AssetState;
}

pub trait InstrumentStateManager<InstrumentKey> {
    type ExchangeKey;
    type AssetKey;
    type Market;

    fn instrument(
        &self,
        key: &InstrumentKey,
    ) -> &InstrumentState<Self::Market, Self::ExchangeKey, Self::AssetKey, InstrumentKey>;

    fn instrument_mut(
        &mut self,
        key: &InstrumentKey,
    ) -> &mut InstrumentState<Self::Market, Self::ExchangeKey, Self::AssetKey, InstrumentKey>;

    fn instruments<'a>(
        &'a self,
        filter: &InstrumentFilter<Self::ExchangeKey, InstrumentKey>,
    ) -> impl Iterator<
        Item = &'a InstrumentState<Self::Market, Self::ExchangeKey, Self::AssetKey, InstrumentKey>,
    >
    where
        Self::Market: 'a,
        Self::ExchangeKey: PartialEq + 'a,
        Self::AssetKey: 'a,
        InstrumentKey: PartialEq + 'a;
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentKey> {
    pub trading: TradingState,
    pub connectivity: ConnectivityStates,
    pub assets: AssetStates,
    pub instruments: InstrumentStates<Market, ExchangeKey, AssetKey, InstrumentKey>,
    pub strategy: Strategy,
    pub risk: Risk,
}

// Todo: probably implement processor here, and move Manager impls elsewhere. Probably with 
//  Manager traits somewhere else, too.

impl<Market, Strategy, Risk, AssetKey, InstrumentKey> ConnectivityManager<ExchangeIndex>
    for EngineState<Market, Strategy, Risk, ExchangeIndex, AssetKey, InstrumentKey>
{
    fn connectivity(&self, key: &ExchangeIndex) -> &ConnectivityState {
        self.connectivity
            .0
            .get_index(key.index())
            .map(|(_key, state)| state)
            .unwrap_or_else(|| panic!("ConnectivityStates does not contain: {key}"))
    }

    fn connectivity_mut(&mut self, key: &ExchangeIndex) -> &mut ConnectivityState {
        self.connectivity
            .0
            .get_index_mut(key.index())
            .map(|(_key, state)| state)
            .unwrap_or_else(|| panic!("ConnectivityStates does not contain: {key}"))
    }
}

impl<Market, Strategy, Risk, AssetKey, InstrumentKey> ConnectivityManager<ExchangeId>
    for EngineState<Market, Strategy, Risk, ExchangeId, AssetKey, InstrumentKey>
{
    fn connectivity(&self, key: &ExchangeId) -> &ConnectivityState {
        self.connectivity
            .0
            .get(key)
            .unwrap_or_else(|| panic!("ConnectivityStates does not contain: {key}"))
    }

    fn connectivity_mut(&mut self, key: &ExchangeId) -> &mut ConnectivityState {
        self.connectivity
            .0
            .get_mut(key)
            .unwrap_or_else(|| panic!("ConnectivityStates does not contain: {key}"))
    }
}

impl<Market, Strategy, Risk, ExchangeKey, InstrumentKey> AssetStateManager<AssetIndex>
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetIndex, InstrumentKey>
{
    fn asset(&self, key: &AssetIndex) -> &AssetState {
        self.assets
            .0
            .get_index(key.index())
            .map(|(_key, state)| state)
            .unwrap_or_else(|| panic!("AssetStates does not contain: {key}"))
    }

    fn asset_mut(&mut self, key: &AssetIndex) -> &mut AssetState {
        self.assets
            .0
            .get_index_mut(key.index())
            .map(|(_key, state)| state)
            .unwrap_or_else(|| panic!("AssetStates does not contain: {key}"))
    }
}

impl<Market, Strategy, Risk, ExchangeKey, InstrumentKey>
    AssetStateManager<ExchangeAsset<AssetNameInternal>>
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetNameInternal, InstrumentKey>
{
    fn asset(&self, key: &ExchangeAsset<AssetNameInternal>) -> &AssetState {
        self.assets
            .0
            .get(key)
            .unwrap_or_else(|| panic!("AssetStates does not contain: {key:?}"))
    }

    fn asset_mut(&mut self, key: &ExchangeAsset<AssetNameInternal>) -> &mut AssetState {
        self.assets
            .0
            .get_mut(key)
            .unwrap_or_else(|| panic!("AssetStates does not contain: {key:?}"))
    }
}

impl<Market, Strategy, Risk, ExchangeKey, AssetKey> InstrumentStateManager<InstrumentIndex>
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentIndex>
{
    type ExchangeKey = ExchangeKey;
    type AssetKey = AssetKey;
    type Market = Market;

    fn instrument(
        &self,
        key: &InstrumentIndex,
    ) -> &InstrumentState<Market, ExchangeKey, AssetKey, InstrumentIndex> {
        self.instruments
            .0
            .get_index(key.index())
            .map(|(_key, state)| state)
            .unwrap_or_else(|| panic!("InstrumentStates does not contain: {key}"))
    }

    fn instrument_mut(
        &mut self,
        key: &InstrumentIndex,
    ) -> &mut InstrumentState<Market, ExchangeKey, AssetKey, InstrumentIndex> {
        self.instruments
            .0
            .get_index_mut(key.index())
            .map(|(_key, state)| state)
            .unwrap_or_else(|| panic!("InstrumentStates does not contain: {key}"))
    }

    fn instruments<'a>(
        &'a self,
        filter: &InstrumentFilter<ExchangeKey, InstrumentIndex>,
    ) -> impl Iterator<Item = &'a InstrumentState<Self::Market, ExchangeKey, AssetKey, InstrumentIndex>>
    where
        Self::Market: 'a,
        ExchangeKey: PartialEq + 'a,
        AssetKey: 'a,
    {
        match filter {
            InstrumentFilter::None => Either::Left(Either::Left(self.instruments.0.values())),
            InstrumentFilter::Exchanges(exchanges) => Either::Left(Either::Right(
                self.instruments
                    .0
                    .values()
                    .filter(|state| exchanges.contains(&state.instrument.exchange)),
            )),
            InstrumentFilter::Instruments(instruments) => Either::Right(
                self.instruments
                    .0
                    .values()
                    .filter(|state| instruments.contains(&state.position.instrument)),
            ),
        }
    }
}

impl<Market, Strategy, Risk, ExchangeKey, AssetKey> InstrumentStateManager<InstrumentNameInternal>
    for EngineState<Market, Strategy, Risk, ExchangeKey, AssetKey, InstrumentNameInternal>
{
    type ExchangeKey = ExchangeKey;
    type AssetKey = AssetKey;
    type Market = Market;

    fn instrument(
        &self,
        key: &InstrumentNameInternal,
    ) -> &InstrumentState<Market, ExchangeKey, AssetKey, InstrumentNameInternal> {
        self.instruments
            .0
            .get(key)
            .unwrap_or_else(|| panic!("InstrumentStates does not contain: {key}"))
    }

    fn instrument_mut(
        &mut self,
        key: &InstrumentNameInternal,
    ) -> &mut InstrumentState<Market, ExchangeKey, AssetKey, InstrumentNameInternal> {
        self.instruments
            .0
            .get_mut(key)
            .unwrap_or_else(|| panic!("InstrumentStates does not contain: {key}"))
    }

    fn instruments<'a>(
        &'a self,
        filter: &InstrumentFilter<ExchangeKey, InstrumentNameInternal>,
    ) -> impl Iterator<
        Item = &'a InstrumentState<Self::Market, ExchangeKey, AssetKey, InstrumentNameInternal>,
    >
    where
        Self::Market: 'a,
        ExchangeKey: PartialEq + 'a,
        AssetKey: 'a,
    {
        match filter {
            InstrumentFilter::None => Either::Left(Either::Left(self.instruments.0.values())),
            InstrumentFilter::Exchanges(exchanges) => Either::Left(Either::Right(
                self.instruments
                    .0
                    .values()
                    .filter(|state| exchanges.contains(&state.instrument.exchange)),
            )),
            InstrumentFilter::Instruments(instruments) => Either::Right(
                self.instruments
                    .0
                    .values()
                    .filter(|state| instruments.contains(&state.position.instrument)),
            ),
        }
    }
}
