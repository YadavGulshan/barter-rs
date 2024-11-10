pub mod action;

// Todo: Probably not, better to have a scriptable Engine rather than some weird docoupling, but:
//   Could add a type Command to Action traits. This could then be added to a Custom Enum
//   that users define, with EngineEvent being the default.

// Decoupled from Auditor and ExecutionTxs
pub struct Engine<State, StrategyT, Risk> {
    pub state: State,
    pub strategy: StrategyT,
    pub risk: Risk,
}
