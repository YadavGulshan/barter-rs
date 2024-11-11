use crate::v2::engine::audit::AuditEvent;

pub mod cancel_orders;
pub mod close_positions;
pub mod generate_algo_orders;
pub mod send_requests;

// Perhaps just Engine "action" methods should be in the new pattern, States can just be event
// processor

pub trait Auditor<Kind> {
    type Snapshot;

    fn snapshot(&self) -> &Self::Snapshot;
    fn send_audit(&mut self, audit: AuditEvent<Kind>);
}

// fn run<Events, Engine, AuditKind>(
//     feed: &mut Events,
//     engine: &mut Engine,
//     auditor: &mut impl Auditor<AuditKind>
// ) -> AuditEvent<AuditKind>
// where
//     Events: Iterator,
//     Events::Item: Clone,
//     Engine: Processor<Events::Item>,
//     Engine::Output: Into<AuditKind>,
// {
//     // Send initial Engine state snapshot
//     // auditor.send_audit()
//
//     // Todo: this appears to be much the same, apart from we now have an Auditor, which I think is
//     //   a good idea.
//
//
// }
