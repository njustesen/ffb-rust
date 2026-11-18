//! The bb2025 `ChompRemovalObserver` hand-off (Java `server/model/change/ChompRemovalObserver`).
//!
//! Java dispatches model-change observers synchronously: the moment `FieldModel.setPlayerState`
//! or `setPlayerCoordinate` drops a chomp, the observer adds `ReportChompRemoved` to the current
//! step's result, so the report sits exactly between the reports written before and after that
//! change. Rust's `FieldModel` cannot reach the `ReportList`, so it queues the update here and
//! `ReportList::add` flushes the queue before the next report (the driver flushes leftovers when
//! the step ends). One game runs per thread, so the thread-local queue is per game.

use std::cell::RefCell;

thread_local! {
    static PENDING: RefCell<Vec<(String, bool)>> = const { RefCell::new(Vec::new()) };
}

/// Queue `(chompeeId, cleared)` updates raised by a FieldModel change.
pub fn push(events: Vec<(String, bool)>) {
    if !events.is_empty() {
        PENDING.with(|p| p.borrow_mut().extend(events));
    }
}

/// Take every queued update, oldest first.
pub fn take() -> Vec<(String, bool)> {
    PENDING.with(|p| std::mem::take(&mut *p.borrow_mut()))
}
