# Tree search with chance nodes on the Rust engine, guided by the g4 outcome model

**Status:** specified 2026-11-26, not started. No code exists for any part of this document.

Paths under `runs/` and the Python files are relative to `ml/game_bc/`; Rust paths are relative to
the workspace root. Line numbers are as of 2026-11-26.

## Context

The cloned policy plays one network call per decision and never looks ahead. Its best checkpoint wins
0.858 against the scripted agent (`runs/g3/epoch_48.pt`); the checkpoint with outcome heads wins 0.804
(`runs/g4_value/epoch_60.pt`). More passes over the 2,224 replays no longer help. The next lever is
search: use the policy as a prior over actions and the outcome heads as the evaluator, in a Monte Carlo
tree search with explicit chance nodes for dice, run on the Rust engine.

## Goal

1. **Beat the policy without search.** The searched agent must win more than the raw policy it
   uses, paired on the same seeds, with the 95% interval of the difference above zero. It must also
   beat the best raw policy we have (g3 end, 0.858).
2. **Get close to 100% against the scripted agent.** Target: 0.95 or more over 1,000 games.

Goal 1 is the test of the search itself. Goal 2 probably needs more than the first search: the
plan reaches for it in stages (search → tuned search → outcome heads and policy retrained on
searched games → search again), and measures the win rate after each stage.

Requested shape:

- PUCT search with a policy prior and a learned evaluator, as in AlphaGo.
- Chance nodes for dice.
- No rollouts. A simulation keeps expanding until it reaches a new chance node.
- Chance nodes form a hierarchy, one per roll (a pass roll, then the catch roll below "accurate").
- A roll with few outcomes → expand all and back up their probability-weighted value. A roll with
  many outcomes (a scatter) → estimate its value without a tree and never search below it.
- Node states are not stored; edges store actions and dice results.
- Heavy parallelism, and GPU batching of state evaluations if it is faster.
- A video of a searched game showing each action's value changing during the search and the
  iteration count.
- The Rust engine does the simulation.

Note on AlphaStar: it has no search. What this plan takes from it is the human-imitation policy as the
prior, which we already have.

## What earlier Blood Bowl search attempts found

| Source | Method | Result |
|---|---|---|
| Justesen et al., CoG 2019 (botbowl paper) | none tested; analysis | ~17 to 30 choices per step, 10 to 40+ steps per turn, 10^30 to 10^51 turns. Random play scored 0 touchdowns in 350,000 games, so random rollouts carry no signal. Python state was too slow to clone. |
| Justesen et al., IEEE ToG 2018 (Hero Academy, the predecessor multi-action game) | vanilla MCTS v non-exploring MCTS v bridge-burning MCTS v evolutionary planning | Vanilla MCTS lost 0 to 100% to the non-exploring variant. Limiting exploration inside a turn is what works. |
| botbowl `mcts_example.py` (docs tutorial) | UCB1, decision and chance nodes, outcomes sampled and grouped by state hash, hand-made evaluator | Against random on the full board, 5 s per decision: 2/20 wins with random rollouts, 7/20 without. |
| Bot Bowl II (2020), Sapling | MCTS with scripted evaluator | 1 win, 20 draws, 39 losses; just above the random bot. |
| Bot Bowl III (2021), Dryad | forward model + MCTS, heuristic reward | 3rd, behind MimicBot (imitation + RL, no search) and GrodBot (scripted). No write-up exists. |
| Humeau et al., CoG 2021 | stochastic planning model of a turn | Only the abstract could be read. |
| Stochastic MuZero (Antonoglou et al., ICLR 2022) | afterstates and chance nodes with a learned evaluator | Method background: this is the node structure used below. |

No published policy-plus-value search for Blood Bowl was found, and no agent that enumerates dice
outcomes. The recurring lessons, and how the plan uses them:

- **Rollouts are worthless** → none; the outcome heads evaluate.
- **The turn is too long for uniform exploration** → a strong prior, only the top few actions per node,
  low exploration constant.
- **State copying was the bottleneck** → Rust engine, about 100 µs per decision.
- **Path-finding macro moves shorten turns** → the policy's move action already picks a square and
  the engine's path finder supplies the route.

## Design

### 1. Engine: fork a running game and script its dice

Neither exists today. Both are additive and do not change behaviour when unused.

- **Fork.** `DriverGameState` (`crates/ffb-engine/src/step/driver.rs:672`) cannot be cloned because
  the step stack holds `Box<dyn Step>`. Add `fn clone_box(&self) -> Box<dyn Step>` to the `Step` trait
  (`step/framework.rs:529`), `#[derive(Clone)]` on every step struct (about 340 files, mechanical;
  fields are plain data), and `Clone` for `DriverStepEntry`, `DriverStepStack`, `GameRng`
  (`ffb-model/src/util/rng.rs:48`) and `DriverGameState`. Add `DriverGameState::fork()`, which also
  drops the event list and report snapshots.
- **Scripted dice.** Every die goes through `GameRng::die(sides)` (`rng.rs:59`). Add an optional
  script: a queue of forced results consumed first, and a trace of `(sides, result)` for every draw.
  With no script the stream is bit-identical to today, so parity gates are unaffected.
- **Known traps.** `ReportList::clone()` returns an empty list (fine for search); the second random
  stream `collections_rng` (prayers, random player picks) is not scripted, so rolls through it are
  treated as closed chance nodes (section 2); two thread-locals (`chomp_observer`, `skill_touch`) are drained per
  step, so each fork is advanced to its next prompt before another is touched on the same thread.
- Clone cost is unmeasured. If the deep-copied `Roster` inside `Team` dominates, share it by `Arc`.

### 2. Chance nodes by re-execution

The engine never announces a roll; one action can roll several dice before the next prompt. So the
outcomes of "state + action" are found by running it:

**One chance node per roll, in a hierarchy.** An action's dice are not flattened into one node.
A pass looks like this:

```
pass action
└─ pass roll
   ├─ accurate ─ catch roll ─┬─ caught            → decision node
   │                         └─ dropped ─ bounce  → few squares: expand
   ├─ inaccurate ─ scatter (8×8×8 squares)        → CLOSED: estimated, never searched
   └─ fumble ─ bounce                             → few squares: expand
```

How the hierarchy is found:

1. Fork, apply the action with an empty script and read the dice trace. An empty trace means the
   action is deterministic.
2. **Rolls.** Consecutive draws that belong to one roll (2d6 armour, the block dice, the three
   scatter dice) form one level. The engine writes one report per roll, so report boundaries in the
   fork give the grouping; a draw without a report is its own level.
3. **Branches of a roll.** Every value of the roll is run with the same forced prefix. Values are
   put in the same branch when they behave the same: same report result (success, failure, the
   block faces shown), same dice that follow, and same end state under two fixed fillers for the
   later dice (all lowest, all highest). The branch probability is the summed probability of its
   values. Example: a 3+ dodge has two branches, 4/6 and 2/6; a pass roll has accurate, inaccurate,
   fumble.
4. **Open or closed.** A roll with few branches (start: at most 12) is **open**: every branch is a
   child, which is either the next roll's chance node or, when the next prompt is reached, a
   decision node. A roll with more branches is **closed**.
5. **Closed nodes are leaves for ever.** The search never descends below one. Its value is
   estimated without a tree: sample outcomes with real dice (start: 24), run each to its next
   prompt, merge identical end states, evaluate them in one batched network call and take the
   weighted mean. Each later visit adds another batch of samples to the mean, up to a cap, so
   important closed nodes get a sharper estimate while staying leaves. If the game itself goes
   into a closed branch, the next decision starts a fresh tree.
6. A whole action's hierarchy is capped (start: 64 leaves); chance nodes beyond the cap are closed.

End states are compared with the existing `state_hash` (`ffb-model/src/util/state_hash.rs:112`).

### 3. Tree and one simulation

- **Nodes hold no game state.** An edge stores its action (plus the scripted agent's answers to
  unmodelled prompts that followed it) and, below a chance node, the dice trace of that outcome.
  A node stores only prior, value, visit counts and state hash. A simulation forks the root once and
  replays its path by applying the stored actions with the stored dice forced (about 100 µs per
  step), so replay is exact and makes no network call. Chance-node enumeration clones the working
  state once per dice run and discards each copy after taking its hash and network request.
  A state cache at the root's children or at turn boundaries is added only if profiling shows the
  replay matters.
- **Decision node:** a prompt for one side. Children are the top actions by prior (start: at most 8,
  covering 95% of the mass). Selection is PUCT, `Q + c · P · sqrt(N) / (1 + n)`. Values are stored
  from the home side's view and read with the sign of the side to move.
- **Chance node:** one per roll, in the hierarchy of section 2.
- **A simulation**
  1. Descend by PUCT. At an open chance node, descend into the branch that is most under-visited
     relative to its probability. A closed chance node ends the simulation: refine its estimate
     with one more sample batch and back that up.
  2. Past the frontier, keep expanding without backing up: each new decision node gets its prior
     from the network and the simulation follows the highest-prior action, through any number of
     dice-free actions, until an action rolls dice.
  3. At that new chance node, build the action's whole roll hierarchy (section 2): open rolls are
     expanded branch by branch down to decision nodes, closed rolls get their sampled estimate.
     All decision-node leaves and all closed-node samples go into one batched network call. The
     value is backed up level by level as `Σ pᵢ · Vᵢ`.
  4. A simulation also ends at a touchdown, the end of a half or the game, or a depth cap.
- **Final choice:** the most visited action at the root. The subtree under the action taken and the
  dice observed is kept for the next decision.
- **Opponent turns** are searched with the same network; a failed roll simply leads to a decision
  node owned by the other side.
- **Prompts the policy does not cover** (re-roll offers, skill use, apothecary, kick-off choices) are
  answered inside the transition by the scripted agent, as `ModelAgent` does today. Re-roll offers
  become a searched two-way decision in a later step.

### 4. Evaluator

The outcome heads give three distributions from the deciding side's view. The value is

`V = w_game·(P(win) − P(lose)) + w_half·(P(outscores) − P(outscored)) + w_drive·(P(own scores next) − P(opponent scores next))`

- Start weights: drive 0.5, half 0.3, match 0.2. On test coaches the drive head is the only one clearly
  better than its prior (1.009 against 1.073) and the match head is worse than its prior (1.084
  against 1.062), so the weights lean on the short horizon. They are tuned in the arena.
- A touchdown sets the drive term to ±1; a finished game uses the real result.
- `g4_value` supplies both prior and value. Its first checkpoint had better test heads than its last
  (match 1.000, half 0.992); both are tried.
- **Risk:** the heads memorised games for most of training. A hand-made evaluator (score, ball
  position and possession, players on the pitch) is built as the control, so a failure can be placed
  on the evaluator or on the search.

### 5. Parallelism

Three levels, all inside the night budget (11 of 16 logical cores, shared with the other agent):

- **Within one decision (tree parallel).** Several worker threads search one shared tree. Each has
  its own working state forked from the root; node statistics are atomics; a virtual loss on the
  path keeps the threads apart. Engine thread-locals are per thread, so the forks do not interfere.
- **Within one chance node.** The dice runs of an enumeration are independent and run on a thread
  pool; so do the per-outcome network requests.
- **Across games.** The arena runs many games at once (`--jobs`), all sending their leaves to one
  evaluation service, which is what fills large batches.

Workers never wait on the network one state at a time: they push leaves to a queue and continue
with another simulation; a batcher thread sends what has gathered and distributes the replies.

### 6. Inference and GPU batching

One call costs about 27 ms on CPU at batch size 1 and the server returns only the chosen action. That
caps a 1 s decision at about 37 evaluations, so:

- New `serve.py` request `{"t":"eval","items":[...]}`: one batched `forward_play(values=True)`,
  returning for each item the top-k actions with probabilities and the 3×3 outcome probabilities.
  The TCP mode gets it too, with the outcome heads it lacks today.
- **Measure first** (existing `FFB_SERVE_PROFILE`): parse, featurise and forward time per item, on
  CPU and on GPU, at batch 1, 8, 32, 128, 512. The earlier GPU server was not faster, but it was fed
  one state per game at a time (batches under 32) and was never profiled. Search produces large
  batches by itself: all outcome children of a chance node plus the leaves of all threads and games.
- **Decision rule:** take whichever gives more evaluations per second at the batch sizes the search
  really produces: one GPU service shared by all games and threads, or several CPU server processes.
- **If the forward pass is fast on GPU but Python featurising is the cost** (it is serial, per
  item): featurise in a pool of processes, or build the tensors in Rust and send them as binary.
  ONNX Runtime with CUDA inside the Rust process is the last resort, since it means porting the
  featuriser.
- Half precision and a compiled model are tried on GPU once the batch path works.

### 7. Video of the search at work

A game played by the searched agent, where each decision shows the search converging.

- **Search log.** The agent writes `<replay stem>.search.<pid>.jsonl`: per decision, about 20
  snapshots taken during the search (iterations 1, 2, 4, 8, … and the last). Each snapshot holds the
  iteration count and, for every root action, type, square, prior, visits and value. The line also
  holds the chosen action and the budget.
- **Rendering.** `policy_video.py` gains a search mode on top of the existing pipeline
  (`watch.py` records the game → plain video → overlay). At each searched decision the board is
  held still for about a second while the snapshots play:
  - on the pitch, the candidate squares are coloured by value (one diverging scale, fixed for the
    whole video) and sized by visits;
  - a side panel lists the top actions with a value bar and visit count that move as the search
    runs, the prior shown as a thin marker for comparison;
  - a counter shows iterations done out of the budget, and the evaluations and tree depth so far;
  - the chosen action is highlighted, then the game continues.
- The outcome-head block (match, half, drive) stays, now showing the root's searched value next to
  the network's own estimate.
- Decisions with a single legal action are not held. With about 370 decisions the video gets about
  six minutes longer; a `--search-hold` option sets the hold time.

### 8. Code layout

| File | Change |
|---|---|
| `crates/ffb-engine/src/step/framework.rs`, all step files | `clone_box`, `#[derive(Clone)]` |
| `crates/ffb-engine/src/step/driver.rs` | `Clone`, `fork()` |
| `crates/ffb-model/src/util/rng.rs` | `Clone`, dice script and trace |
| `crates/ffb-parity/src/model_agent.rs` | split `answer` into "build request and legal set" and "map action index to `Action`", so the search can list actions with priors; add agent name `search` to `enum Player` and the arena name match (lines 469 to 492, 619 to 625) |
| `crates/ffb-parity/src/search/` (new: `chance.rs`, `tree.rs`, `eval.rs`) | enumeration, PUCT tree, evaluator and batching client |
| `ml/game_bc/serve.py` | `eval` request in both stdin and TCP modes |
| `ml/game_bc/arena.py` | pairing `search` with budget flags |
| `ml/game_bc/watch.py`, `policy_video.py` | pass the search log prefix; search mode of the overlay |

Reused as is: `state_json` / `roster_json` / `reach_json` (`decision_record.rs`), `mde::find_routes`
for move paths, `HeuristicAgent` for unmodelled prompts, `state_hash`, the arena loop and its
`ARENA` result lines, `policy_video.py` for watching a searched game.

## Steps

1. **Fork and dice script** in the engine, with tests. Day work except the full test run.
2. **Chance enumeration** with tests on known rolls.
3. **Batched `eval` request**, profiled on CPU and GPU; pick the faster service by the decision rule.
4. **Search agent** in the arena, single-threaded first (the reference for correctness), then tree
   parallel with the batcher: prior + outcome heads, then the hand-made evaluator as control.
4b. **Search video** of one human v orc game, frames checked, opened.
5. **Night measurements** (below), then tuning of weights, child count, budgets, exploration constant.
6. **Close the gaps the first search leaves:** searched re-roll offers (the scripted agent answers
   them until then), the roll-target shortcut for enumeration, and the g3 policy as prior with the
   `g4_value` heads as evaluator (two networks) if g3's prior proves better.
7. **Improvement loop towards 100%,** repeated while the win rate still rises:
   - Play searched games against the scripted agent and against itself (several thousand per round,
     over nights), recording every searched decision with the root visit counts and the real
     outcome of the game, half and drive.
   - Retrain from the current checkpoint: the play head on the visit counts, the outcome heads on
     the real outcomes. This gives the heads far more games than the 2,224 replays they memorised,
     and games from the states the search actually reaches. Human replays stay in the mix.
   - Measure the new raw policy and the new searched agent on the fixed 1,000 seeds.

## Verification

- **Fork:** fork a game at many prompts of 20 seeded games, play the same actions on both copies with
  the same RNG state; every state hash matches.
- **No behaviour change:** `cargo test --workspace`, then a small mirror parity gate (20 seeds per
  edition, foreground, at night) with the script unused.
- **Roll hierarchy:** branch probabilities sum to 1 at every level; a 3+ dodge gives 4/6 and 2/6; a
  two-dice block gives the right face-pair probabilities; a pass gives accurate → catch (caught,
  dropped), inaccurate closed, fumble; forced dice reproduce the same state twice; for small
  hierarchies the leaf probabilities equal a brute-force enumeration of all dice.
- **Closed nodes:** a test asserts no node is ever created below one.
- **Search sanity:** with one simulation and no exploration the agent picks the policy's top action,
  so its win rate must match raw `g4_value` (0.804) within error.
- **Strength, paired on the same seeds against the scripted agent:**
  search(`g4_value`) against raw `g4_value`, against raw g3 (0.858), and against search with the
  hand-made evaluator. 200 games each first (95% interval about ±0.05), 1,000 only for the best
  setting. One searched game at 0.25 s per decision is about 1.5 minutes, so 200 games is about
  5 core-hours and 1,000 games is a full night on 11 cores.
- **Goal check after every stage:** paired difference to the raw policy (goal 1) and the win rate
  over the fixed 1,000 seeds against 0.95 (goal 2), both with 95% intervals. Losses and draws of the
  best agent are saved as replays and looked at, since near 100% the remaining games show what is
  still missing.
- **Speed report:** forks per second, enumeration runs per chance node, share of open and closed
  chance nodes, evaluations per second by batch size, simulations and tree depth per decision.
- **Parallel search:** with the same budget in iterations, the parallel agent's win rate matches the
  single-threaded one within error; reported speed-up in iterations per second by thread count.
- **GPU or CPU:** the evaluations-per-second table by batch size for both, and which one was chosen.
- **Video:** frames from three searched decisions checked by eye (values move between snapshots,
  the iteration counter rises to the budget, the highlighted action is the one played), then the
  video opened.

## Constraints kept

- No compute between 08:00 and 20:00: release builds, test runs, arena games and profiling run at
  night, at most 70% of the machine, resumable, stopping themselves before 08:00.
- The Java-ported engine logic is not changed; the additions are `Clone`, `fork()` and an unused-by-
  default dice script. Nothing in `C:\Users\Admin\niels\ffb` is touched.
- Nothing is committed until asked.

## Risks

- **Weak outcome heads.** If search with them does not beat the raw policy while search with the
  hand-made evaluator does, the next step is better heads (more replays, or training on searched
  games), not more search.
- **Inference cost.** If batching gives little, the search stays shallow; the profile in step 3
  decides whether Rust-side inference is needed.
- **Branch grouping is inferred, not declared by the engine.** Two values of a roll that differ only
  under dice the two fillers do not show would be merged wrongly. The roll-hierarchy tests check the grouping
  on every roll kind met in 200 games against a brute-force enumeration.
- **Closed nodes are sampled estimates,** so their value is noisy; the sample cap bounds the noise.
- **Search leaves the human-like states** the heads were trained on, so their estimates get worse
  the deeper the tree; the depth cap and low exploration limit this.
