mod market_event_updater;
pub mod trading_state_updater;
// Todo: I think ultimately this module "state" would actually be at equal level to "action",
//     ie/ an Engine "actions", but State "updates"
//   '--> Actions generate Outputs, Updaters generate Audits???
//     Strategy, Risk could be special case, Processors? Or Actions such as ProcessMarketEvent
//     MarketState would just be a XUpdater

// Todo: update from market, then try run loop!!!!!!1
