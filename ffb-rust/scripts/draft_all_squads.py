"""Re-draft EVERY parity squad in data/teams/<edition>/ from the drafting rules.

Why this exists
---------------
The squads were hand-drafted once (2026-08-08) and checked against
docs/PARITY_COVERAGE_REQUIREMENTS.md sec.8/sec.11. That check passed, and the squads were still
illegal, because nothing in that document required legality under the RULEBOOK -- sec.11's money
formula was, in its own words, "DERIVED FROM THE DATA, not assumed", so it reproduced the squads
it was meant to audit. Seven systematic defects survived it, including star players paid for with
neither gold nor Skill Points and ZERO Sideline Staff on all 111 squads (the spec had no field for
them, and gen_java_parity_data.py hardcoded <cheerleaders>0</cheerleaders>).

docs/PARITY_COVERAGE_REQUIREMENTS.md sec.19 now states those rules as R6, with a citation into
rules/ for every constant. THIS script drafts to R6 and asserts it before writing anything.

What is re-drafted and what is not
----------------------------------
COMPOSITION is drafted from scratch: players, re-rolls, apothecary, fans, assistant coaches,
cheerleaders, and all the money. IDENTITY is read from the existing spec and preserved -- which
squad files exist, and each one's `race` and `roster_id`. Identity is not a drafting decision: the
file set IS the parity matrix's cell set, each file being one gate (ffb-parity's `--home <stem>`),
and the roster a squad belongs to is a property of the cell, not of the draft.

The purchase order
------------------
Specified by the user, with the residual tie-break decided here and documented:

  1. one of EVERY positional the caps allow            (this is also what satisfies R2/R4)
  2. more players, DEAREST first, while the squad can still reach 12 bodies and 2 re-rolls
  3. cheapest available player up to 12 players        ("at least 11 when first drafted")
  4. team re-rolls, 2 of them
  5. apothecary, if the roster may hire one; then a 3rd re-roll if affordable
  6. a 13th body, then a last dearest-first upgrade pass with whatever remains
  7. fans to the edition's drafting maximum
  8. assistant coaches to 6, then cheerleaders to 6    (drains the 10,000 units)
  9. more players up to 16, then re-rolls up to 8
 10. anything that CANNOT be spent is lost

Step 2 is what makes these look like teams rather than like minimum-compliance squads. Buying
the CHEAPEST filler first and only then trying to upgrade produced caricatures: Chaos Chosen
came out with 2 Chosen Blockers and 10 Beastmen, Khemri with one Tomb Guardian and 13
Skeletons. A coach buys the good players first and fills the rest with linemen, so that is what
the drafter does -- bounded by `can_still_reach_floor`, which refuses any purchase that would
leave the squad unable to field 12 players with 2 re-rolls.

Step 9 is the rule, not a shortfall: "all the gold pieces a team has must be spent when drafting
your team. Any gold pieces not spent are lost" (rules/core_rules/06_matched_play.md), and the same
sentence appears for exhibition play. `treasury` therefore records lost gold, and the script
asserts it is smaller than every remaining legal purchase -- i.e. that nothing further COULD have
been bought.

Group Big Guy caps
------------------
"A Chaos Chosen team may have a single Big Guy, chosen from the following" is a cap on what the
team may CONTAIN, across a GROUP of positions -- which the per-position `quantity` field cannot
express. It is read from the official team page, and only from there: a roster with no official
page for that ruleset (the FUMBBL legacy imports) has no group cap, and its per-position
quantities govern alone. Where a cap makes "one of every positional" unsatisfiable in one squad,
the cell has R3 variant squads and each one's forced picks are named in GROUP_PICKS below; R2 is
then met by the UNION over the cell (R4).

Usage
-----
  python scripts/draft_all_squads.py            # re-draft and write
  python scripts/draft_all_squads.py --check    # draft in memory, diff, write nothing
  python scripts/draft_all_squads.py --report   # per-squad spend breakdown

Then ALWAYS:
  python scripts/validate_teams.py --selftest
  python scripts/validate_teams.py --r5
  python scripts/gen_java_parity_data.py        # or the two engines field different teams
"""

import argparse
import json
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(Path(__file__).resolve().parent))
from validate_teams import PAGES  # noqa: E402  official team pages, per ruleset

EDITIONS = ("bb2016", "bb2020", "bb2025")
BUDGET = 1_100_000

# ── R6 constants. Every one of these is quoted in PARITY_COVERAGE_REQUIREMENTS.md sec.19 ──
MIN_PLAYERS = 11          # "at least 11 players ... when it is first drafted"
MAX_PLAYERS = 16          # "may never have more than 16 players"
FLOOR_PLAYERS = 12        # this script's floor inside 11..16: 11 is the rule, but a squad
                          # with no bench plays a man short after one casualty
BENCH_TARGET = 13         # this script's choice inside 11..16, not a rule
MAX_REROLLS = 8           # "a maximum of 8 Team Re-rolls"
APOTHECARY_COST = 50_000
STAFF_COST = 10_000       # assistant coaches and cheerleaders alike
MAX_COACHES = 6
MAX_CHEERLEADERS = 6
STAR_TYPES = ("Star", "Infamous Staff")
BIG_GUY_TYPES = {"bigguy"}   # data/rosters spells the type both "Big Guy" and "BigGuy"

# Fans differ per edition, and getting this wrong is defect 4/5 of sec.19.
#   field, start, max, cost-per-improvement, counts-in-TV
FANS = {
    # CRP Fan Factor: 0-9 at 10,000, and it DOES count in Team Value (UtilTeamValue:
    # teamValue += pTeam.getFanFactor() * 10000).
    "bb2016": ("fan_factor", 0, 9, 10_000, True),
    # Exhibition play: "a team drafted for exhibition play will have a Dedicated Fans
    # characteristic of 0 ... can still improve this up to a maximum of 6, at a cost of
    # 10,000 gold pieces per improvement".
    "bb2020": ("dedicated_fans", 0, 6, 10_000, False),
    # Matched play uses the League drafting rule: starts at 1, "up to a maximum of 3", 5,000
    # per improvement.
    "bb2025": ("dedicated_fans", 1, 3, 5_000, False),
}

# Group Big Guy cap wording on the official pages -> the cap.
CAP_WORDS = {"a single": 1, "up to two": 2, "up to three": 3, "up to four": 4}

# For a cell whose group cap forbids fielding every Big Guy at once, which group members each
# squad file takes. The union over a cell must cover every group member (R4). Derived from the
# page caps; the split matches the cell set these files have always had, so the gates stay
# comparable and `coverage_squad_tests` keeps testing the same squad for the same positional.
GROUP_PICKS = {
    ("bb2020", "chaos"):                       ["chaos.minotaur"],
    ("bb2020", "chaos_chaosogre"):             ["chaos.chaosogre"],
    ("bb2020", "chaos_chaostroll"):            ["chaos.chaostroll"],
    ("bb2020", "old_world_alliance_ogre"):     ["oldworldalliance.ogre"],
    ("bb2020", "old_world_alliance_treeman"):  ["oldworldalliance.altern_forest_treeman"],
    ("bb2020", "renegades"):                   ["37731", "37732", "37733"],
    ("bb2020", "renegades_37730"):             ["37730", "37731", "37733"],
    ("bb2020", "underworld"):                  ["37844"],
    ("bb2020", "underworld_underworldtroll"):  ["37844.underworldtroll"],
    ("bb2025", "chaos"):                       ["chaos.minotaur"],
    ("bb2025", "chaos_ogre"):                  ["chaos.ogre"],
    ("bb2025", "chaos_troll"):                 ["chaos.troll"],
    ("bb2025", "old_world_alliance_ogre"):     ["oldworldalliance.ogre"],
    ("bb2025", "old_world_alliance_treeman"):  ["oldworldalliance.altern_forest_treeman"],
    ("bb2025", "renegades"):                   ["37730", "37731", "37732"],
    ("bb2025", "renegades_37733"):             ["37730", "37731", "37733"],
    ("bb2025", "underworld"):                  ["underworld.troll.warpstone"],
    ("bb2025", "underworld_37844"):            ["37844"],
}


def load(path: Path):
    with open(path, encoding="utf-8") as fh:
        return json.load(fh)


def rosters(edition: str) -> dict:
    out = {}
    for p in sorted((ROOT / "data" / "rosters" / edition).glob("roster_*.json")):
        r = load(p)
        out[r["id"]] = r
    return out


def group_cap(edition: str, race: str):
    """The official page's group Big Guy cap, or None when the page says nothing / there is no
    official page for this (edition, race). No page means no group cap: the per-position
    `quantity` limits govern on their own."""
    page_dir, mapping = PAGES[edition]
    slug = mapping.get(race)
    if not slug:
        return None
    page = page_dir / f"{slug}.md"
    if not page.exists():
        return None
    text = page.read_text(encoding="utf-8", errors="replace")
    m = re.search(r"may (?:have|include|take) (%s) Big Guy" % "|".join(CAP_WORDS), text)
    return CAP_WORDS[m.group(1)] if m else None


def is_big_guy(pos: dict) -> bool:
    return pos.get("type", "").lower().replace(" ", "") in BIG_GUY_TYPES


def draftable(roster: dict) -> list:
    """Positions a coach may hire. Star / Infamous Staff positions are Inducements, never
    drafted (sec.19 defect 1)."""
    return [p for p in roster["positions"] if p.get("type") not in STAR_TYPES]


class Draft:
    """One squad's purchases, with the money recomputed from scratch on every change."""

    def __init__(self, edition, stem, race, roster, cap, picks):
        self.edition, self.stem, self.race = edition, stem, race
        self.roster, self.cap, self.picks = roster, cap, picks
        self.positions = draftable(roster)
        self.by_id = {p["id"]: p for p in self.positions}
        self.counts = Counter()
        self.rerolls = 0
        self.apothecaries = 0
        self.coaches = 0
        self.cheerleaders = 0
        field, start, _mx, _cost, _tv = FANS[edition]
        self.fans = start
        self.fan_field = field

    # ── money ────────────────────────────────────────────────────────────────────────────
    @property
    def player_cost(self):
        return sum(self.by_id[pid]["cost"] * n for pid, n in self.counts.items())

    @property
    def staff_cost(self):
        return (self.rerolls * self.roster["reroll_cost"]
                + self.apothecaries * APOTHECARY_COST
                + (self.coaches + self.cheerleaders) * STAFF_COST)

    @property
    def team_value(self):
        """UtilTeamValue.findTeamValue, 1:1: re-rolls + fanFactor + coaches + cheerleaders +
        apothecaries + player costs. Dedicated Fans are NOT in TV; Fan Factor is."""
        tv = self.player_cost + self.staff_cost
        _f, _s, _m, cost, in_tv = FANS[self.edition]
        if in_tv:
            tv += self.fans * cost
        return tv

    @property
    def spent(self):
        _f, start, _m, cost, in_tv = FANS[self.edition]
        fans = 0 if in_tv else (self.fans - start) * cost
        return self.team_value + fans

    @property
    def left(self):
        return BUDGET - self.spent

    @property
    def n_players(self):
        return sum(self.counts.values())

    # ── legality of one more purchase ────────────────────────────────────────────────────
    def room(self, pid, for_swap=False) -> bool:
        """Is one more of this position legal? Per-position quantity, the 16-player maximum,
        and the group Big Guy cap. `for_swap` skips the squad-size test, for an upgrade that
        removes one player as it adds another."""
        pos = self.by_id[pid]
        if self.counts[pid] + 1 > pos["quantity"]:
            return False
        if not for_swap and self.n_players + 1 > MAX_PLAYERS:
            return False
        if self.cap is not None and is_big_guy(pos):
            fielded = sum(n for q, n in self.counts.items() if is_big_guy(self.by_id[q]))
            if fielded + 1 > self.cap:
                return False
        return True

    def buy(self, pid, n=1):
        for _ in range(n):
            assert self.room(pid), f"{self.stem}: illegal purchase {pid}"
            assert self.by_id[pid]["cost"] <= self.left, f"{self.stem}: cannot afford {pid}"
            self.counts[pid] += 1

    def affordable(self):
        """Positions we could legally buy one more of right now, cheapest first. Deterministic:
        ties break on the position id."""
        opts = [p for p in self.positions
                if self.room(p["id"]) and p["cost"] <= self.left]
        return sorted(opts, key=lambda p: (p["cost"], p["id"]))

    def cheapest_fill_cost(self, need):
        """Cost of the `need` cheapest additional players still legal, or None if there are not
        that many slots left. Quantity and group caps are respected, so this is a real price,
        not `need * min(cost)`."""
        slots = []
        for pos in sorted(self.positions, key=lambda p: (p["cost"], p["id"])):
            room = pos["quantity"] - self.counts[pos["id"]]
            if room <= 0:
                continue
            if self.cap is not None and is_big_guy(pos):
                fielded = sum(n for q, n in self.counts.items() if is_big_guy(self.by_id[q]))
                room = min(room, max(0, self.cap - fielded))
            slots.extend([pos["cost"]] * room)
        slots.sort()
        if len(slots) < need:
            return None
        return sum(slots[:need])

    def can_still_reach_floor(self):
        """Could this squad still reach FLOOR_PLAYERS bodies and its team re-rolls with the gold
        it has left? Guards the dearest-first buying in step 2 from spending the squad into a
        corner -- an ST5 Big Guy is worth having, but not at the price of a legal squad.

        The re-rolls are RESERVED here, not bought later out of the leftovers: the specified
        order is positionals and the 11-player minimum, then 2-3 re-rolls, then more players.
        Letting the dearest-first pass spend first dropped almost every squad to 2 re-rolls and
        took the apothecary off 70 of them."""
        left = self.left - self.reroll_reserve(3) - self.apothecary_reserve()
        if left < 0:
            return False
        # MIN_PLAYERS, not FLOOR_PLAYERS: 11 is the rule and therefore the invariant. The 12th
        # body is a preference, applied in step 3 only where it does not cost a re-roll --
        # bb2016 renegades cannot have both (one of each of its 10 positionals is 895,000, and
        # 12 players plus 2 re-rolls does not fit in 1,100,000).
        need = max(0, MIN_PLAYERS - self.n_players)
        if need == 0:
            return True
        fill = self.cheapest_fill_cost(need)
        return fill is not None and left >= fill

    def reroll_reserve(self, want):
        """Gold that must be held back for `want` team re-rolls."""
        cap_rr = min(MAX_REROLLS, self.roster.get("max_rerolls", MAX_REROLLS))
        return max(0, min(want, cap_rr) - self.rerolls) * self.roster["reroll_cost"]

    def apothecary_reserve(self):
        """Gold held back for the apothecary, where the roster may hire one.

        Reserved rather than bought out of the leftovers. "More players or staff" leaves the
        order to this script, and both realism and coverage point the same way: no coach who
        may hire an apothecary goes without one, and letting the dearest-first pass spend first
        cut the apothecary from 70 squads to 15, taking most of the apothecary mechanic's
        exercise off the matrix with it."""
        if self.apothecaries or not self.roster.get("apothecary"):
            return 0
        return APOTHECARY_COST

    def cheapest_purchase(self):
        """The cheapest thing of ANY kind still legal, or None. Used to prove that leftover gold
        genuinely could not be spent."""
        opts = []
        aff = [p for p in self.positions if self.room(p["id"])]
        if aff:
            opts.append(min(p["cost"] for p in aff))
        if self.rerolls < min(MAX_REROLLS, self.roster.get("max_rerolls", MAX_REROLLS)):
            opts.append(self.roster["reroll_cost"])
        if self.apothecaries == 0 and self.roster.get("apothecary"):
            opts.append(APOTHECARY_COST)
        _f, _s, mx, cost, _tv = FANS[self.edition]
        if self.fans < mx:
            opts.append(cost)
        if self.coaches < MAX_COACHES or self.cheerleaders < MAX_CHEERLEADERS:
            opts.append(STAFF_COST)
        return min(opts) if opts else None

    def upgrade(self):
        """Turn spare gold into better players by swapping a cheap body for the dearest
        positional that still has room.

        Without this, "one of every positional, then fill with the cheapest lineman" drafts
        caricatures: Khemri came out with one Tomb Guardian, one Blitz-ra, one Thro-ra and 13
        Skeletons, which satisfies R2 to the letter and is not a team anyone would field. It is
        also worse for coverage -- a single Tomb Guardian rolls Mighty Blow a fraction as often
        as the four a coach would take.

        Never removes the LAST copy of a position, so R2 survives every swap; the squad size
        and every cap are unchanged by construction. Deterministic: dearest target first,
        cheapest victim first, ties on the position id."""
        while True:
            targets = sorted((p for p in self.positions if self.room(p["id"], for_swap=True)),
                             key=lambda p: (-p["cost"], p["id"]))
            victims = sorted((p for p in self.positions if self.counts[p["id"]] >= 2),
                             key=lambda p: (p["cost"], p["id"]))
            for t in targets:
                swapped = False
                for v in victims:
                    if t["id"] == v["id"]:
                        continue
                    delta = t["cost"] - v["cost"]
                    if delta <= 0 or delta > self.left:
                        continue
                    self.counts[v["id"]] -= 1
                    if self.counts[v["id"]] == 0:
                        del self.counts[v["id"]]
                    self.counts[t["id"]] += 1
                    swapped = True
                    break
                if swapped:
                    break
            else:
                return

    # ── the draft ────────────────────────────────────────────────────────────────────────
    def run(self):
        # 1. one of EVERY positional the caps allow. Group members only as GROUP_PICKS says,
        #    so the cell's variants between them cover the whole group.
        for pos in self.positions:
            if self.cap is not None and is_big_guy(pos) and pos["id"] not in self.picks:
                continue
            self.buy(pos["id"])

        # 2. spend on the BEST players first: repeatedly buy the dearest position that still
        #    has room, but only while the squad can still reach its player floor and 2
        #    re-rolls afterwards. This is the step that makes the squads look like teams.
        while True:
            for pos in sorted((p for p in self.positions
                               if self.room(p["id"]) and p["cost"] <= self.left),
                              key=lambda p: (-p["cost"], p["id"])):
                self.counts[pos["id"]] += 1
                if self.can_still_reach_floor():
                    break
                self.counts[pos["id"]] -= 1
            else:
                break

        # 3. fill to the 11-player minimum, cheapest first -- this one is the rule.
        while self.n_players < MIN_PLAYERS:
            opts = self.affordable()
            assert opts, f"{self.stem}: cannot reach {MIN_PLAYERS} players"
            self.buy(opts[0]["id"])

        # 3b. and to 12 where it does not cost a re-roll. A squad on exactly 11 plays a man
        #     short after one casualty, but two re-rolls matter more than the 12th body.
        while self.n_players < FLOOR_PLAYERS:
            opts = [o for o in self.affordable()
                    if o["cost"] <= self.left - self.reroll_reserve(2)
                    - self.apothecary_reserve()]
            if not opts:
                break
            self.buy(opts[0]["id"])

        # 4. team re-rolls, 2 first. Never fewer -- a team with no re-roll cannot exercise the
        #    team re-roll, Loner, or block re-roll paths at all, and the block re-roll is the
        #    mechanic BACKLOG sec.H.14 had just made reachable.
        cap_rr = min(MAX_REROLLS, self.roster.get("max_rerolls", MAX_REROLLS))
        while self.rerolls < min(2, cap_rr) and self.roster["reroll_cost"] <= self.left:
            self.rerolls += 1
        assert self.rerolls >= min(2, cap_rr), f"{self.stem}: could not afford 2 re-rolls"

        # 5. apothecary, where the roster may hire one -- BEFORE the third re-roll. Two
        #    re-rolls and an apothecary is the standard shape of a drafted team, and buying
        #    the third re-roll first left 57 of the 94 eligible rosters without one.
        if self.roster.get("apothecary") and APOTHECARY_COST <= self.left:
            self.apothecaries = 1

        # 5b. the third re-roll, taken after the player floor and the apothecary are secured
        #     rather than before them (buying it first left 18 squads on exactly 11 players).
        while self.rerolls < min(3, cap_rr) and self.roster["reroll_cost"] <= self.left:
            self.rerolls += 1

        # 6. a bench, cheapest first. BEFORE the fans: fans are bought in 5,000/10,000 units
        #    and would otherwise eat the price of a body (bb2016 chaos drafted 11 players and
        #    Fan Factor 8 when this ran the other way round).
        while self.n_players < BENCH_TARGET:
            opts = self.affordable()
            if not opts:
                break
            self.buy(opts[0]["id"])

        # 6b. a last dearest-first pass: swap a cheap body for the dearest positional that
        #     still has room, wherever the remaining gold covers the difference.
        self.upgrade()

        # 7. fans to the drafting maximum.
        _f, _s, mx, fan_cost, _tv = FANS[self.edition]
        while self.fans < mx and fan_cost <= self.left:
            self.fans += 1

        # 8. Sideline Staff -- the ruleset's own advice for leftover gold, and the first time
        #    any parity squad has had a single member of staff.
        while self.coaches < MAX_COACHES and STAFF_COST <= self.left:
            self.coaches += 1
        while self.cheerleaders < MAX_CHEERLEADERS and STAFF_COST <= self.left:
            self.cheerleaders += 1

        # 9. whatever the 10,000-unit staff could not absorb: more players, then more
        #    re-rolls. Both are legal to the last gold piece, and leaving gold on the table
        #    is not (it is simply lost).
        while self.n_players < MAX_PLAYERS:
            opts = self.affordable()
            if not opts:
                break
            self.buy(opts[0]["id"])
        while self.rerolls < cap_rr and self.roster["reroll_cost"] <= self.left:
            self.rerolls += 1

        self.check()
        return self

    # ── R6 assertions, before anything is written ────────────────────────────────────────
    def check(self):
        n = self.n_players
        assert MIN_PLAYERS <= n <= MAX_PLAYERS, f"{self.stem}: {n} players"
        for pid, c in self.counts.items():
            assert c <= self.by_id[pid]["quantity"], f"{self.stem}: {c}x {pid} over quantity"
        if self.cap is not None:
            bgs = sum(c for pid, c in self.counts.items() if is_big_guy(self.by_id[pid]))
            assert bgs <= self.cap, f"{self.stem}: {bgs} Big Guys over group cap {self.cap}"
        cap_rr = min(MAX_REROLLS, self.roster.get("max_rerolls", MAX_REROLLS))
        assert 0 <= self.rerolls <= cap_rr, f"{self.stem}: {self.rerolls} re-rolls"
        assert self.apothecaries in (0, 1)
        assert not (self.apothecaries and not self.roster.get("apothecary")), \
            f"{self.stem}: apothecary on a roster that may not hire one"
        assert 0 <= self.coaches <= MAX_COACHES and 0 <= self.cheerleaders <= MAX_CHEERLEADERS
        _f, _s, mx, _c, _tv = FANS[self.edition]
        assert 0 <= self.fans <= mx, f"{self.stem}: fans {self.fans} over {mx}"
        assert self.spent <= BUDGET, f"{self.stem}: spent {self.spent} over budget"
        # Nothing further could have been bought -- this is what "all gold must be spent" means
        # once the remainder is smaller than every legal purchase.
        cheapest = self.cheapest_purchase()
        assert cheapest is None or self.left < cheapest, \
            f"{self.stem}: {self.left} left but {cheapest} would still buy something"

    # ── output ───────────────────────────────────────────────────────────────────────────
    def spec(self, special_rules):
        """The team_<stem>.json body.

        Jersey numbers matter for coverage, not just for tidiness: both harnesses field the
        first 11 players by number (canonical_setup_action / ParityRunner.placeReserves), so a
        positional numbered 12+ never takes the pitch and has NO parity evidence -- the same
        failure R2 exists to prevent. Numbering therefore goes ROUND BY ROUND: the first copy
        of every position in roster order, then the second copies, and so on. The 8 positionals
        of a 16-man Underworld squad get shirts 1-8 rather than 12-16, which is what emitting
        each position's copies consecutively did (it benched the Warpstone Troll, the Gutter
        Runner, the Blitzer and the Thrower all at once)."""
        order = []
        for copy in range(max(self.counts.values(), default=0)):
            for idx, pos in enumerate(self.positions):
                if self.counts[pos["id"]] > copy:
                    order.append((copy, idx, pos["id"]))
        players = [{"nr": i + 1, "position_id": pid} for i, (_c, _i, pid) in enumerate(order)]
        out = {
            "edition": self.edition,
            "race": self.race,
            "roster_id": self.roster["id"],
            "rerolls": self.rerolls,
            "reroll_cost": self.roster["reroll_cost"],
            "apothecaries": self.apothecaries,
            "assistant_coaches": self.coaches,
            "cheerleaders": self.cheerleaders,
            "dedicated_fans": self.fans if self.fan_field == "dedicated_fans" else 0,
            "fan_factor": self.fans if self.fan_field == "fan_factor" else 0,
            "treasury": self.left,
            "spent": self.spent,
            "team_value": self.team_value,
            "players": players,
            "special_rules": special_rules,
        }
        return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--check", action="store_true", help="draft and diff, write nothing")
    ap.add_argument("--report", action="store_true", help="print a spend breakdown per squad")
    ap.add_argument("--edition", choices=EDITIONS, action="append")
    args = ap.parse_args()

    editions = args.edition or list(EDITIONS)
    changed = same = 0
    lost_total = 0
    for ed in editions:
        ros = rosters(ed)
        team_dir = ROOT / "data" / "teams" / ed
        for path in sorted(team_dir.glob("team_*.json")):
            stem = path.stem[len("team_"):]
            old = load(path)
            race, rid = old["race"], old["roster_id"]
            roster = ros.get(rid)
            if roster is None:
                print(f"C1 CRITICAL {ed}/{path.name}: roster_id {rid!r} does not resolve")
                return 2
            cap = group_cap(ed, race)
            picks = GROUP_PICKS.get((ed, stem), [])
            if cap is not None and not picks:
                bg = [p["id"] for p in draftable(roster) if is_big_guy(p)]
                if len(bg) > cap:
                    print(f"MISSING GROUP_PICKS for {ed}/{stem}: cap {cap} over {len(bg)} "
                          f"Big Guys {bg}")
                    return 2
                picks = bg
            d = Draft(ed, stem, race, roster, cap, picks).run()
            spec = d.spec(old.get("special_rules") or [])
            lost_total += d.left
            if args.report:
                print(f"{ed} {stem:32s} {d.n_players:2d}p rr{d.rerolls} apo{d.apothecaries} "
                      f"fans{d.fans} ac{d.coaches} cl{d.cheerleaders} "
                      f"tv={d.team_value:>9,} spent={d.spent:>9,} lost={d.left:>6,}")
            if json.dumps(spec, sort_keys=True) == json.dumps(old, sort_keys=True):
                same += 1
                continue
            changed += 1
            if not args.check:
                path.write_text(json.dumps(spec, indent=2) + "\n", encoding="utf-8")

    verb = "would change" if args.check else "rewrote"
    print(f"\n{verb} {changed} squads, {same} unchanged; {lost_total:,} gold unspendable in total")
    return 0


if __name__ == "__main__":
    sys.exit(main())
