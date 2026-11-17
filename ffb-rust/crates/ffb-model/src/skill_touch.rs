//! Skill-touch telemetry (BACKLOG §M phase 0) — Rust-only, no Java twin.
//!
//! Counts, per `SkillId`, how often an ENGINE skill check came back true for a player who has
//! the skill (`Player::has_skill`, `has_skill_property*`, `has_unused_skill_with_property`,
//! `skill_id_with_property`). It gives the "silent" skills — Block, Guard, Tackle, Sprint, which
//! the report stream never names — a count, so the coverage ledger can tell "fielded and live"
//! from "fielded and never consulted".
//!
//! Parity already proves the two engines behave identically, so Java needs no counterpart.
//! Off by default (one relaxed atomic load per check); the parity runner enables it, and wraps
//! every agent decision in [`suppress`] so the agents' own look-ahead does not count.
//! Purely observational: it never changes a return value.

use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::enums::SkillId;

const SLOTS: usize = 512;

static ENABLED: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// Slot = `SkillId as usize`; the id is stored beside its count so `take` can name it.
    static COUNTS: RefCell<Vec<(Option<SkillId>, u64)>> = RefCell::new(vec![(None, 0); SLOTS]);
    static SUPPRESS: Cell<u32> = const { Cell::new(0) };
}

/// Turn counting on or off for the whole process.
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

#[inline]
fn active() -> bool {
    ENABLED.load(Ordering::Relaxed) && SUPPRESS.with(|s| s.get() == 0)
}

/// Record one touch of `id` (no-op unless enabled and not suppressed).
#[inline]
pub fn touch(id: SkillId) {
    if !active() {
        return;
    }
    let i = id as usize;
    if i < SLOTS {
        COUNTS.with(|c| {
            let slot = &mut c.borrow_mut()[i];
            slot.0 = Some(id);
            slot.1 += 1;
        });
    }
}

/// Record a touch of every skill in `ids` (a property check: each matching skill was consulted).
#[inline]
pub fn touch_all(ids: impl Iterator<Item = SkillId>) {
    if !active() {
        return;
    }
    COUNTS.with(|c| {
        let mut c = c.borrow_mut();
        for id in ids {
            let i = id as usize;
            if i < SLOTS {
                c[i].0 = Some(id);
                c[i].1 += 1;
            }
        }
    });
}

/// RAII guard: while alive, touches on this thread are not counted.
pub struct Suppress(());

impl Drop for Suppress {
    fn drop(&mut self) {
        SUPPRESS.with(|s| s.set(s.get() - 1));
    }
}

/// Suppress counting until the returned guard drops (agent decisions).
pub fn suppress() -> Suppress {
    SUPPRESS.with(|s| s.set(s.get() + 1));
    Suppress(())
}

thread_local! {
    static FIELDED: RefCell<Vec<SkillId>> = const { RefCell::new(Vec::new()) };
}

/// Record that a fielded player carries `id` (the ledger's "fielded" column). Always on.
pub fn field(id: SkillId) {
    FIELDED.with(|f| {
        let mut f = f.borrow_mut();
        if !f.contains(&id) {
            f.push(id);
        }
    });
}

/// Drain the fielded set.
pub fn take_fielded() -> Vec<SkillId> {
    FIELDED.with(|f| std::mem::take(&mut *f.borrow_mut()))
}

/// Drain this thread's counts: `(skill, count)` for every non-zero slot, then reset.
pub fn take() -> Vec<(SkillId, u64)> {
    COUNTS.with(|c| {
        let mut c = c.borrow_mut();
        let out = c.iter().filter_map(|&(id, n)| id.filter(|_| n > 0).map(|id| (id, n))).collect();
        c.iter_mut().for_each(|v| *v = (None, 0));
        out
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_only_when_enabled_and_unsuppressed() {
        set_enabled(false);
        touch(SkillId::Block);
        set_enabled(true);
        touch(SkillId::Block);
        {
            let _g = suppress();
            touch(SkillId::Block);
        }
        touch_all([SkillId::Block, SkillId::Dodge].into_iter());
        let got = take();
        set_enabled(false);
        assert!(got.contains(&(SkillId::Block, 2)));
        assert!(got.contains(&(SkillId::Dodge, 1)));
        assert!(take().is_empty());
    }
}
