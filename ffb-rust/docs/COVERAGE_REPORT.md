# Parity mechanic-coverage report

How to see which game mechanics a parity suite actually exercised — the actions, dice
rolls, injuries, kickoff events, and every `GameEvent` type emitted — so you can spot the
thin spots and know what a tier has (and hasn't) proven.

The counts come from the **Rust engine's** per-seed event logs. Because every seed in a
green tier reproduces stock Java's per-step state hashes exactly, the action / roll /
outcome tallies are trustworthy as *shared* coverage. Caveat: the event **stream** itself
is not part of the compared state hash, so treat the catalog as "what the engine did,"
verified indirectly by parity rather than event-by-event against Java.

## Where the data lives

Each parity run writes one events log per seed:

```
parity/<home>_vs_<away>/seed_<N>_rust_events.jsonl
```

Every line is one `GameEvent` as JSON with a `"type"` field (plus event-specific fields
like `"action"`, `"result"`, `"success"`, `"was_ko"`, …). Running a seed range (or each
seed individually) regenerates these files.

## Procedure — aggregate the tallies

From `ffb-rust/ffb-rust`, after the suite has been run so the `*_rust_events.jsonl` files
are fresh (swap `lineman_vs_lineman` for the tier you're reporting on):

```bash
DIR=parity/lineman_vs_lineman
cat $DIR/seed_*_rust_events.jsonl | tr -cd '[:print:]\n' > /tmp/allev.txt
echo "TOTAL events: $(wc -l < /tmp/allev.txt)"

# Event types (the full catalog), most→least frequent
grep -oE '"type":"[^"]+"' /tmp/allev.txt | sed 's/"type":"//;s/"//' | sort | uniq -c | sort -rn

# Player actions declared, by action type
grep '"type":"playerAction"' /tmp/allev.txt | grep -oE '"action":"[^"]+"' | sort | uniq -c | sort -rn

# Kickoff events, by result
grep '"type":"kickoffResultEvent"' /tmp/allev.txt | grep -oE '"result":"[^"]+"' | sort | uniq -c | sort -rn

# Roll pass/fail (dodgeRoll, pickupRoll, catchRoll all carry "success":true/false)
for t in dodgeRoll pickupRoll catchRoll; do
  ok=$(grep "\"type\":\"$t\"" /tmp/allev.txt | grep -c '"success":true')
  no=$(grep "\"type\":\"$t\"" /tmp/allev.txt | grep -c '"success":false')
  echo "$t: pass=$ok fail=$no"
done

# passRoll by result / distance; blockRoll by nr_of_dice
grep '"type":"passRoll"'  /tmp/allev.txt | grep -oE '"result":"[^"]+"'    | sort | uniq -c
grep '"type":"passRoll"'  /tmp/allev.txt | grep -oE '"distance":"[^"]+"'  | sort | uniq -c
grep '"type":"blockRoll"' /tmp/allev.txt | grep -oE '"nr_of_dice":[0-9-]+' | sort | uniq -c

# Injuries: KO / casualty / lasting
echo "ko=$(grep '"type":"injury"' /tmp/allev.txt | grep -c '"was_ko":true')  \
cas=$(grep '"type":"injury"' /tmp/allev.txt | grep -c '"was_cas":true')  \
serious=$(grep '"type":"injury"' /tmp/allev.txt | grep -c '"serious_injury":"')"

# Sanity: any re-roll actually consumed? (parity agent declines all → expect 0)
grep -c '"rerolled":true' /tmp/allev.txt
```

`strings` is unavailable on this box — the `tr -cd '[:print:]\n'` strips the stray control
bytes some logs carry so `grep` behaves.

## Procedure — render the report page

`docs/coverage_report.html` is a self-contained, theme-aware template (one snapshot lives
in it as an example — the lineman tier, 100 seeds). To publish a fresh one:

1. Run the aggregation above and collect the numbers.
2. Copy `docs/coverage_report.html` to the scratchpad, update: the masthead subtitle +
   summary-strip stats, each section's rows (`data-c` on `td.count` drives the bar, the
   text is the visible number), and the `catalog[]` array in the `<script>` (all event
   types, `[name, count, tier]` where tier ∈ `core|cov|rare`). Chip rule of thumb:
   `≥100 → core`, `10–99 → covered`, `≤9 → rare`.
3. Publish with the `Artifact` tool (favicon 🏈). It's a private work-product page; the
   user shares it from the page if they want.

Bars are linear within each table (largest row = full width) except the full catalog,
which is √-scaled so the long tail stays visible — the number is always the ground truth.

## What a skill-less lineman tier does NOT cover

Star players, named skills (Block, Dodge, Sure Hands, Pass, Catch…), weather beyond
Sweltering Heat's fainting, and actual **re-roll consumption** (the agent always declines).
Those await a richer-roster tier (e.g. full human vs full human).
