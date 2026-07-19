# Phase AI Plan — Close AH's Follow-Ups, Then Full Sweep of the 3 Largest Unswept Pools

*Written 2026-07-19, immediately after Phase AH (17,396 tests).*

## Context: where we were

Phase AH closed all 5 of Phase AG's named follow-ups plus one more fresh-audit round, and found
real bugs in most of what it checked — the third consecutive phase to do so. Its closing note named:

1. A tracker-mapping bug: 20 `bb2020/special/*.java` rows pointed at dead, uncompiled Rust code.
2. 3 pre-existing/documented modifier-factory gaps (Decay, armor-modifier branches, GFI card mods).
3. A missing `HasReRollProperties` behavior on one dialog parameter file.
4. Three large pools only partially swept: `ffb-mechanics/modifiers/` (25 of 77 sampled),
   dialog parameters (24 of ~71 sampled), `step/generator/` (20 of 114 sampled).
5. A recommendation to keep running the fresh re-verification technique — it had not gone dry
   after 3 consecutive phases.

Parity/integration testing stayed explicitly out of scope per the user's instruction — this phase
was unit-test-only, prioritizing test coverage over any other workstream.

## What actually happened

**Stage 1 (foreground, 3 quick wins):** all 3 closed. The tracker mapping was fixed (20 rows
re-pointed to the real compiled files, dead directory left in place pending a user decision). The
3 modifier-factory gaps were re-verified: 2 confirmed still genuinely deferred/inert, but the third
(`go_for_it_modifier_factory.rs`) turned out to be a **stale claim** — it's fully wired and tested,
not inert. The `HasReRollProperties` gap was fixed, and a second file with the identical gap
(`DialogReRollPropertiesParameter`) was found and fixed alongside it.

**Stage 2 (10 parallel isolated-worktree agents):** rather than sampling a fraction of each pool
again, this phase fully swept all 3 named pools to completion — all 78 modifier files, all 67
dialog-parameter files, all 120 generator files. 6 of the first 10 worktree-creation attempts hit
lock contention from concurrent `git worktree add` calls (145+ stale worktrees already existed in
`.claude/worktrees/` from prior sessions) and had to be retried sequentially; one retry batch was
initially missed and had to be caught and relaunched after a user check-in. One agent's worktree
also transiently vanished mid-run before it could commit — its described fix was independently
recovered by inspecting the worktree's uncommitted diff directly and committing it before the
worktree was cleaned up, so no work was lost.

**Result: 7 of 10 batches found real, confirmed bugs** (20 bugs total across the phase, full
breakdown in `SESSION.md`'s Phase AI entry). 3 batches came back fully clean: all 34 top-level
`step/generator/` files (confirmed architecturally — these are abstract base-class data holders,
not sequence-building logic), all 16 `bb2016/` generator files, and 1 of 3 dialog-parameter
batches.

**Stage 3:** merged all 10 worktree branches into `main` (all fast-forward or clean 3-way merges,
no conflicts), ran the full workspace test/clippy suite after each merge, updated
`TRANSLATION_TRACKER.md` and `SESSION.md`, and committed/pushed.

## Estimated progress after Phase AI

- **File/method coverage:** unchanged, ~99.8–99.9%+ (0 `○`/`~` rows in the tracker).
- **Tests:** 17,396 → 17,423 (+27).
- **Behavioral-correctness residual:** still present, still no sign of drying up. This is the
  fourth consecutive phase (AF, AG, AH, AI) where fresh re-verification found real bugs in most of
  what it checked, even after this phase deliberately targeted the largest previously-unswept
  pools to completion rather than sampling. **The project is not behaviorally done**, despite
  file-coverage looking complete — the audit technique keeps surfacing real drift.
- **What's left:** (1) a user decision on the dead `bb2020/special/` directory; (2) scope a fresh
  area for the next phase, since the 3 largest named pools are now fully swept — no more
  "partially-sampled" pools remain from prior phases' backlog, so the next phase needs new
  scoping (candidates: skill files not covered by the constructor-arg-drift sweep, `ffb-client`
  dialog-side handlers, `ffb-server` handler bodies); (3) parity/integration testing remains the
  only large, entirely out-of-scope workstream, unstarted by design.
