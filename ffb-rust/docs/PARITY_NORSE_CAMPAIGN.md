# Norse — heuristic-agent parity campaign (8/9)

**Status 2026-09-05: ONE gate open** (bb2020 @0, a single seed). Started after nurgle
(`117a532cd`).

## Surface

Lineman (Block, Drunkard, Thick Skull, **Unsteady**), Beer Boar (Dodge, **No Ball**, Pick-me-up,
Stunty, **Titchy**), Berserker (Block, **Frenzy**, Jump Up), Thrower (Catch, Dauntless, Pass,
**Strip Ball**), Ulfwerener (Frenzy, Unsteady), Snow Troll (Claws, **Disturbing Presence**, Frenzy,
Loner 4, **Unchannelled Fury**).

Frenzy on three positions plus a negatrait (Unchannelled Fury) — the mandatory-follow-up and
second-block machinery is heavily exercised here.

## Baseline (measured on `e025e6f0d`, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | 99 (seed 90) | **100** |
| bb2025 | **100** | **100** | **100** |

Random controls: bb2016 **100/100**, bb2020 **100/100**, bb2025 **100/100**.

**Eight of nine green at baseline**, and the one red is a single seed at argmax — so a
resolution/content divergence, not a draw-count split.

## The open red — bb2020 @0 seed 90, localised

`first_state_divergence.sh` puts the resolving activation at **i=87** (`home_03, Block`), which both
engines declare identically. The state at i=88 differs in exactly **three tokens**:

```
R  h1 t8/7 active=home   a03:7,0,Stunned   h02:8,1,Standing(inactive)
J  h1 t8/8 active=away   a03:8,1,Standing  h02:8,2,Standing(active)
```

So on the same block: Rust pushes `a03` to **(7,0)** — the top sideline — and knocks it **Stunned**,
with the attacker following up to (8,1) and home's turn continuing; Java leaves `a03` **Standing at
(8,1)**, its attacker at (8,2), and the turn passes to away.

That is a pushback-direction / block-result divergence at the sideline, in the same neighbourhood as
the nippon chain-crowd-push fix (`8790f9838`) but **not** the same fault — that one removed the
wrong player from a chain push into the crowd, whereas here nobody leaves the pitch and the
knockdown itself differs.

**Next step**: dump the pushback squares for that block on both sides (`FFB_PBSQ`-style probe, and
Java's `JAVA_PUSHBACK` line already exists), then the block dice and the chosen die index. Do NOT
infer from the GFI or dice counts alone — see the necromantic ledger for five wrong turns taken that
way.

## Not yet done

Coverage harvest, and of course the ninth gate. This race is NOT closed.
