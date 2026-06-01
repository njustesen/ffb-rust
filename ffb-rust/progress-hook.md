# SESSION RESUME PROTOCOL
# Read this file at the start of EVERY work session.

## Steps to Resume

1. **Read `progress.html`** — find current state:
   - What tier/component was last in progress?
   - How many parity seeds have passed?
   - Any recorded failures?

2. **Read `SESSION.md`** — find in-progress notes from last session:
   - Which component was being worked on?
   - Were there failing tests to fix?
   - Is a TDD fix loop in progress?

3. **Continue from exactly where the last session left off.**
   - If a component was partially implemented, finish it first.
   - If tests were failing, fix them before moving on.
   - If parity was stopped at seed N, resume from seed N.

4. **After every component:**
   - Update the checklist in `progress.html`
   - Update `SESSION.md` with current state

5. **After every parity seed:**
   - Update the parity table row in `progress.html`
   - Update `SESSION.md` if a failure occurred

## Component Workflow (for each of the 529 components)

1. Write comprehensive Java test suite (all code paths; must pass; never modify production Java)
2. Translate tests to Rust
3. Implement component in Rust; iterate until all Rust tests pass
4. Update `progress.html` checklist row
5. Update `SESSION.md`

## Parity TDD Loop (when logs diverge)

1. Stop at the failing seed
2. Print first divergent event pair (Java vs Rust)
3. Write focused Java unit test for the failing scenario
4. Translate that Java test to Rust
5. Fix Rust engine until Rust test passes (never modify production Java)
6. Verify all existing Rust tests still pass
7. Restart parity from seed 1

## Plan Reference

Full plan: `C:\Users\Admin\.claude\plans\make-a-plan-for-silly-sparrow.md`
Java project: `C:\Users\Admin\niels\ffb\`
Rust project: `C:\Users\Admin\niels\ffb-rust\`
Java AI module reference: https://github.com/njustesen/ffb (ffb-ai/)
