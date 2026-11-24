"""Setup records -> model inputs, the placement loop and its rules.

A record (written by `ffb-replay setups` / `ffb-parity --setup-scenario`, see
crates/ffb-parity/src/setup_record.rs) is one snapshot of a game at a kick-off setup.

`build_proto` turns it into the static part of an episode, in CANONICAL coordinates: the side
setting up is always on the left (x 0..12, line of scrimmage x = 12), whatever side it really
plays on. `Episode` is the dynamic part: the bench pointer and the board as players are placed
one at a time. The same Episode drives training (teacher forcing towards a human formation) and
inference (the model's own rollout).

Actions: square index x*15+y (0..389), SKIP (390: leave this player on the bench), END (391:
never legal -- a setup ends by rule when the team has fielded all it can).
"""
from __future__ import annotations

import numpy as np

W, H = 26, 15
N_SQ = W * H
A_SKIP, A_END = N_SQ, N_SQ + 1
N_ACT = N_SQ + 2

TEAM_MAX = 24          # players of one team in the player table
P_MAX = 2 * TEAM_MAX   # player table rows
K_MAX = 16             # skills per player
BENCH = 16             # bench slots shown to the model, per team
MAX_FIELD, MIN_LOS, MAX_WIDE = 11, 3, 2
LOS_X = 12

N_FEATS = 12
N_GLOBALS = 37

WEATHERS = ["nice", "sunny", "rain", "blizzard", "heat"]
EDITIONS = ["bb2016", "bb2020", "bb2025"]


def sq(x: int, y: int) -> int:
    return x * H + y


def xy(s: int) -> tuple[int, int]:
    return s // H, s % H


def is_los(s: int) -> bool:
    x, y = xy(s)
    return x == LOS_X and 4 <= y <= 10


def wide_zone(s: int) -> int:
    """0 = top wide zone, 1 = bottom wide zone, -1 = neither."""
    y = s % H
    return 0 if y <= 3 else (1 if y >= 11 else -1)


def _targets(p: dict, edition: str) -> tuple[int, int]:
    """(AG, PA) as roll targets (lower is better; 99 = no passing stat)."""
    ag = p["ag"]
    if edition == "bb2016":
        ag = 7 - ag  # bb2016 agility is "higher is better"
    pa = p["pa"] if p["pa"] and p["pa"] > 0 else 99
    return ag, pa


def class_key(p: dict) -> tuple:
    """Players with the same key are interchangeable (names and numbers ignored)."""
    return (p["cost"], p["ma"], p["st"], p["ag"], p["pa"], p["av"], tuple(p["skills"]),
            p["niggling"], tuple(sorted(p["injuries"])))


def bench_key(p: dict, edition: str) -> tuple:
    """Bench order: value, ST, AG, PA, AV, number of skills (best first), then the class."""
    ag, pa = _targets(p, edition)
    return (-p["cost"], -p["st"], ag, pa, -p["av"], -len(p["skills"]), repr(class_key(p)))


def player_feats(p: dict, edition: str) -> list[float]:
    ag, pa = _targets(p, edition)
    return [
        p["ma"] / 9.0, p["st"] / 6.0, (7 - ag) / 6.0,
        0.0 if pa == 99 else (7 - pa) / 6.0, 0.0 if pa == 99 else 1.0,
        p["av"] / 11.0, p["cost"] / 200000.0, float(p["niggling"]), float(len(p["injuries"])),
        float(p["big_guy"]), float(p["lineman"]), len(p["skills"]) / 8.0,
    ]


def build_vocab(records) -> dict[str, int]:
    """Skill name -> id. 0 = padding, 1 = unknown skill."""
    names = sorted({s for r in records for p in r["players"] for s in p["skills"]})
    return {n: i + 2 for i, n in enumerate(names)}


class Proto:
    """The static part of one setup, in canonical coordinates."""
    __slots__ = ("rec_id", "kind", "offense", "feats", "skills", "bench", "classes", "target",
                 "opp_board", "opp_bench", "globals", "n_field", "ids", "setting_home", "heuristic", "z")


def _team_vars(team: dict, players: list[dict]) -> list[float]:
    st = [p["state"] for p in players]
    avail = sum(1 for p in players if p["state"] == "RESERVE" or p["x"] is not None)
    return [
        team["rerolls"] / 8.0, float(team["apothecaries"]), team["bribes"] / 3.0,
        team["team_value"] / 2_000_000.0, avail / 16.0, st.count("KO") / 5.0,
        (st.count("BH") + st.count("SI") + st.count("RIP")) / 5.0, st.count("BANNED") / 3.0,
    ]


def build_proto(rec: dict, vocab: dict[str, int], side: str | None = None, check: bool = True):
    """Record -> (Proto, None), or (None, reason) when the formation is not a legal one.

    `side` ("home"/"away") overrides the record's setting side (the viewer sets up both teams of
    one scenario record). With `check=False` no human formation is expected (inference)."""
    setting_home = rec["setting_home"] if side is None else side == "home"
    me, them = ("home", "away") if setting_home else ("away", "home")
    edition = rec["edition"]
    cx = (lambda x: x) if setting_home else (lambda x: W - 1 - x)

    own = [p for p in rec["players"] if p["side"] == me][:TEAM_MAX]
    opp = [p for p in rec["players"] if p["side"] == them][:TEAM_MAX]
    table = own + opp

    pr = Proto()
    pr.rec_id = (rec["replay_id"], rec["command"])
    pr.kind = rec["kind"]
    pr.setting_home = setting_home
    pr.heuristic = None
    pr.z = 0.5             # how the game ended for this team: 1 won, 0.5 drew, 0 lost (outcomes.py)
    pr.ids = [p["id"] for p in table]
    pr.feats = np.zeros((P_MAX, N_FEATS), np.float32)
    pr.skills = np.zeros((P_MAX, K_MAX), np.int64)
    for i, p in enumerate(table):
        pr.feats[i] = player_feats(p, edition)
        for k, s in enumerate(p["skills"][:K_MAX]):
            pr.skills[i, k] = vocab.get(s, 1)

    avail = [i for i, p in enumerate(own) if p["state"] == "RESERVE" or p["x"] is not None]
    avail.sort(key=lambda i: bench_key(own[i], edition))
    pr.bench = avail
    cls: dict[tuple, int] = {}
    pr.classes = [cls.setdefault(class_key(own[i]), len(cls)) for i in avail]
    pr.n_field = min(MAX_FIELD, len(avail))

    # the human formation: class id -> squares
    pr.target = {}
    squares = []
    for i, c in zip(avail, pr.classes):
        p = own[i]
        if p["x"] is not None:
            s = sq(cx(p["x"]), p["y"])
            pr.target.setdefault(c, []).append(s)
            squares.append(s)

    opp_on = [(len(own) + j, p) for j, p in enumerate(opp) if p["x"] is not None]
    pr.opp_board = [(sq(cx(p["x"]), p["y"]), i) for i, p in opp_on]
    opp_res = [(len(own) + j, p) for j, p in enumerate(opp) if p["x"] is None and p["state"] == "RESERVE"]
    opp_res.sort(key=lambda ip: bench_key(ip[1], edition))
    pr.opp_bench = [i for i, _ in opp_res][:BENCH]

    # the opponent is on the pitch only when we are the receiving (offense) side
    pr.offense = bool(opp_on) if rec["kind"] == "scenario" else rec["kind"] == "offense"

    if check:
        if len(squares) != pr.n_field:
            return None, f"fielded {len(squares)} of {pr.n_field}"
        if len(set(squares)) != len(squares):
            return None, "two players on one square"
        if any(xy(s)[0] > LOS_X for s in squares):
            return None, "player outside the own half"
        if sum(map(is_los, squares)) < min(MIN_LOS, pr.n_field):
            return None, "line of scrimmage short"
        if any(sum(1 for s in squares if wide_zone(s) == z) > MAX_WIDE for z in (0, 1)):
            return None, "wide zone over-full"
        if any(xy(s)[0] <= LOS_X for s, _ in pr.opp_board):
            return None, "opponent in our half"

    my, op = rec[me], rec[them]
    weather = rec["weather"].lower()
    g = [
        float(pr.offense), float(rec["half"] == 2), float(rec["half"] >= 3),
        my["turn"] / 8.0, op["turn"] / 8.0, min(my["score"], 6) / 4.0, min(op["score"], 6) / 4.0,
        max(-4, min(4, my["score"] - op["score"])) / 4.0,
    ]
    g += [float(w in weather) for w in WEATHERS]
    g += [float(edition == e) for e in EDITIONS]
    g += _team_vars(my, own) + _team_vars(op, opp)
    pr.globals = np.array(g + [0.0] * 5, np.float32)  # last 5: the episode's progress
    assert len(pr.globals) == N_GLOBALS
    return pr, None


class Episode:
    """One setup in progress: players leave the bench in order, each is placed or skipped."""

    def __init__(self, proto: Proto, flip_y: bool = False):
        self.p = proto
        self.flip = flip_y
        self.rest = list(range(len(proto.bench)))   # bench positions still undecided, in bench order
        self.board = np.zeros(N_SQ, np.int64)       # table row + 1, 0 = empty
        for s, row in proto.opp_board:
            self.board[self._f(s)] = row + 1
        self.placed: list[tuple[int, int]] = []     # (table row, square)
        self.los = 0
        self.wide = [0, 0]
        self.remaining = {c: [self._f(s) for s in ss] for c, ss in proto.target.items()}

    def _f(self, s: int) -> int:
        if not self.flip:
            return s
        x, y = xy(s)
        return sq(x, H - 1 - y)

    @property
    def done(self) -> bool:
        return len(self.placed) >= self.p.n_field or not self.rest

    @property
    def cur_row(self) -> int:
        """Player-table row of the player at the head of the bench."""
        return self.p.bench[self.rest[0]]

    def legal_mask(self) -> np.ndarray:
        m = np.zeros(N_ACT, bool)
        slots = self.p.n_field - len(self.placed)
        bench_left = len(self.rest)
        los_need = max(0, min(MIN_LOS, self.p.n_field) - self.los)
        for x in range(LOS_X + 1):
            for y in range(H):
                s = sq(x, y)
                if self.board[s]:
                    continue
                z = wide_zone(s)
                if z >= 0 and self.wide[z] >= MAX_WIDE:
                    continue
                if slots <= los_need and not is_los(s):
                    continue
                m[s] = True
        m[A_SKIP] = bench_left > slots
        return m

    def consistent_mask(self, j: int | None = None) -> np.ndarray:
        """Actions that agree with the human formation for bench position `j` (default: the head
        of the bench)."""
        m = np.zeros(N_ACT, bool)
        c = self.p.classes[self.rest[0] if j is None else j]
        rem = self.remaining.get(c, [])
        m[rem] = True
        same_left = sum(1 for k in self.rest if self.p.classes[k] == c)
        m[A_SKIP] = same_left > len(rem)
        return m

    def obs(self) -> dict[str, np.ndarray]:
        p = self.p
        own_bench = np.zeros(BENCH, np.int64)
        rest = [p.bench[j] for j in self.rest[:BENCH]]
        own_bench[:len(rest)] = np.array(rest, np.int64) + 1
        opp_bench = np.zeros(BENCH, np.int64)
        opp_bench[:len(p.opp_bench)] = np.array(p.opp_bench, np.int64) + 1
        g = p.globals.copy()
        slots = p.n_field - len(self.placed)
        g[-5:] = [len(self.placed) / 11.0, slots / 11.0, len(self.rest) / 16.0,
                  self.los / 3.0, (self.wide[0] + self.wide[1]) / 4.0]
        return {
            "feats": p.feats, "skills": p.skills, "board": self.board.copy(),
            "cur": np.int64(self.cur_row + 1), "own_bench": own_bench, "opp_bench": opp_bench,
            "globals": g, "legal": self.legal_mask(), "head": np.int64(p.offense),
            "z": np.float32(getattr(p, "z", 0.5)),
        }

    def step(self, action: int, j: int | None = None) -> None:
        """Decides bench position `j` (default: the head of the bench): a square, or A_SKIP."""
        if action == A_END:
            raise ValueError("END is never a legal action")
        j = self.rest[0] if j is None else j
        if action != A_SKIP:
            row = self.p.bench[j]
            self.board[action] = row + 1
            self.placed.append((row, action))
            self.los += is_los(action)
            z = wide_zone(action)
            if z >= 0:
                self.wide[z] += 1
            rem = self.remaining.get(self.p.classes[j])
            if rem and action in rem:
                rem.remove(action)
        self.rest.remove(j)

    def formation(self) -> list[tuple[int, int]]:
        """(class id, square) of every placed player, un-flipped."""
        cls = dict(zip(self.p.bench, self.p.classes))
        return sorted((cls[row], self._f(s)) for row, s in self.placed)


def human_formation(proto: Proto) -> list[tuple[int, int]]:
    return sorted((c, s) for c, ss in proto.target.items() for s in ss)


def teacher_steps(proto: Proto, rng: np.random.Generator, flip_y: bool = False):
    """One random human-consistent trajectory: a list of (obs, consistent-and-legal mask)."""
    ep = Episode(proto, flip_y)
    out = []
    while not ep.done:
        o = ep.obs()
        target = ep.consistent_mask() & o["legal"]
        choices = np.flatnonzero(target)
        if len(choices) == 0:
            raise AssertionError(f"{proto.rec_id}: no legal action agrees with the human formation")
        o["target"] = target
        out.append(o)
        ep.step(int(rng.choice(choices)))
    return out


def subset_steps(proto: Proto, rng: np.random.Generator, flip_y: bool = False):
    """Like `teacher_steps`, but every step is its own random partial formation: a random subset
    of the bench has already been decided the way the human did (in any order), the rest is still
    on the bench in bench order, and the model decides the head of that bench. Shows the model many
    more boards per formation than the bench-order prefixes (which are the only boards it meets
    when it sets up on its own)."""
    out = []
    n = len(proto.bench)
    for _ in range(proto.n_field):
        ep = Episode(proto, flip_y)
        for j in rng.permutation(n)[:rng.integers(0, n)]:
            if len(ep.placed) >= proto.n_field - 1:
                break                       # keep at least one square to fill
            ep.step(int(rng.choice(np.flatnonzero(ep.consistent_mask(int(j))))), int(j))
        o = ep.obs()
        o["target"] = ep.consistent_mask() & o["legal"]
        if o["target"].any():
            out.append(o)
    return out


def stack(observations: list[dict]) -> dict[str, np.ndarray]:
    return {k: np.stack([o[k] for o in observations]) for k in observations[0]}
