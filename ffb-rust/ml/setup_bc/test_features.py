"""pytest ml/setup_bc -- the placement loop, its rules and the set-valued target."""
import json
import os

import numpy as np
import pytest

from features import (A_END, A_SKIP, LOS_X, Episode, bench_key, build_proto, build_vocab, class_key,
                      human_formation, is_los, sq, teacher_steps, wide_zone, xy)
from rollout import compare

DATA = os.path.join(os.path.dirname(os.path.abspath(__file__)), "data", "setups.jsonl")


def player(pid, side, x=None, y=None, cost=50000, st=3, skills=(), state=None, **kw):
    p = {"id": pid, "side": side, "nr": 1, "position_id": "t.p", "ma": 6, "st": st, "ag": 3, "pa": 4, "av": 9,
         "skills": list(skills), "cost": cost, "niggling": 0, "injuries": [], "big_guy": False, "lineman": True,
         "state": state or ("STANDING" if x is not None else "RESERVE"), "x": x, "y": y}
    p.update(kw)
    return p


def record(players, setting_home=True, kind="defense"):
    team = {"race": "T", "team_value": 1000000, "rerolls": 3, "apothecaries": 1, "bribes": 0, "turn": 0, "score": 0}
    return {"replay_id": "t", "command": 1, "drive": 0, "kind": kind, "setting_home": setting_home,
            "edition": "bb2025", "half": 1, "weather": "Nice Weather", "home": team, "away": team, "players": players}


def legal_home_formation():
    """3 on the line, 2 per wide zone, 4 behind: 11 linemen + 1 blitzer on the bench + 1 lineman."""
    squares = [(12, 6), (12, 7), (12, 8), (11, 1), (11, 2), (11, 12), (11, 13), (9, 5), (9, 7), (9, 9), (6, 7)]
    ps = [player(f"h{i}", "home", x, y) for i, (x, y) in enumerate(squares)]
    ps[0].update(cost=85000, skills=["Block"], lineman=False)      # the blitzer, on the line at (12,6)
    ps.append(player("h11", "home"))                               # a twelfth lineman, left out
    ps.append(player("h12", "home", state="KO"))                   # not available
    ps += [player(f"a{i}", "away") for i in range(11)]
    return ps


def test_bench_order_and_classes():
    a = player("a", "home", cost=50000)
    b = player("b", "home", cost=85000, skills=["Block"])
    c = player("c", "home", cost=50000, st=4)
    order = sorted([a, b, c], key=lambda p: bench_key(p, "bb2025"))
    assert [p["id"] for p in order] == ["b", "c", "a"]             # value first, then strength
    assert class_key(a) == class_key(player("z", "home", cost=50000, nr=99))
    assert class_key(a) != class_key(c)


def test_proto_canonicalises_the_away_side_to_the_left():
    ps = legal_home_formation()
    for p in ps:                                                   # mirror: the away team now holds that formation
        p["side"] = "away" if p["side"] == "home" else "home"
        if p["x"] is not None:
            p["x"] = 25 - p["x"]
    proto, why = build_proto(record(ps, setting_home=False), {"Block": 2})
    assert why is None
    assert all(xy(s)[0] <= LOS_X for _, s in human_formation(proto))
    assert sum(is_los(s) for _, s in human_formation(proto)) == 3


def test_illegal_formations_are_rejected():
    ps = legal_home_formation()
    ps[1]["x"], ps[1]["y"] = 10, 7                                 # only two left on the line
    assert build_proto(record(ps), {})[1] == "line of scrimmage short"
    ps = legal_home_formation()
    ps[7]["x"], ps[7]["y"] = 10, 0                                 # a third player in the top wide zone
    assert build_proto(record(ps), {})[1] == "wide zone over-full"
    ps = legal_home_formation()
    ps[10]["x"], ps[10]["y"], ps[10]["state"] = None, None, "RESERVE"   # ten fielded with twelve available
    assert build_proto(record(ps), {})[1].startswith("fielded")


def test_consistent_set_treats_identical_players_as_one():
    proto, _ = build_proto(record(legal_home_formation()), {"Block": 2})
    ep = Episode(proto)
    # head of the bench is the blitzer (highest value): exactly one square, no skip
    m = ep.consistent_mask()
    assert list(np.flatnonzero(m)) == [sq(12, 6)]
    ep.step(sq(12, 6))
    # then eleven linemen for ten squares: any remaining lineman square, or skip
    m = ep.consistent_mask()
    assert m.sum() == 11 and m[A_SKIP] and not m[sq(12, 6)]
    assert ep.legal_mask()[A_SKIP] and not ep.legal_mask()[A_END]
    ep.step(A_SKIP)
    m = ep.consistent_mask()                                      # ten linemen, ten squares: no more skipping
    assert m.sum() == 10 and not m[A_SKIP]
    assert not ep.legal_mask()[A_SKIP]


def test_mask_forces_the_line_and_closes_full_wide_zones():
    ps = [player(f"h{i}", "home") for i in range(11)] + [player(f"a{i}", "away") for i in range(11)]
    proto, _ = build_proto(record(ps), {}, check=False)
    ep = Episode(proto)
    assert ep.legal_mask()[:390].sum() == 13 * 15
    ep.step(sq(5, 0)); ep.step(sq(5, 1))
    m = ep.legal_mask()
    assert not any(m[s] for s in range(390) if wide_zone(s) == 0)  # top wide zone is full
    assert m[sq(5, 12)]
    for y in range(4, 10):
        ep.step(sq(3, y))                                          # 8 placed, none on the line, 3 slots left
    m = ep.legal_mask()
    assert m[:390].sum() == 7 and all(is_los(s) for s in np.flatnonzero(m[:390]))
    for y in (4, 5, 6):
        ep.step(sq(12, y))
    assert ep.done


def test_random_partial_formations_keep_a_legal_human_consistent_target():
    from features import subset_steps
    proto, _ = build_proto(record(legal_home_formation()), {"Block": 2})
    rng = np.random.default_rng(3)
    counts = set()
    for _ in range(40):
        for o in subset_steps(proto, rng):
            assert o["target"].any() and not (o["target"] & ~o["legal"]).any()
            counts.add(int((o["board"] > 0).sum()))
    assert len(counts) >= 8                                        # boards of many different fill levels


def test_compare_ignores_names_and_the_top_bottom_mirror():
    human = [(0, sq(12, 6)), (1, sq(12, 7)), (1, sq(10, 2))]
    assert compare(human, human)["exact"] == 1.0
    mirrored = [(0, sq(12, 8)), (1, sq(12, 7)), (1, sq(10, 12))]
    assert compare(mirrored, human)["exact"] == 1.0
    swapped = [(1, sq(12, 6)), (0, sq(12, 7)), (1, sq(10, 2))]     # right squares, wrong classes on two
    r = compare(swapped, human)
    assert r["exact"] == 0.0 and r["shape_overlap"] == 1.0 and r["displacement"] == 0.0


@pytest.mark.skipif(not os.path.exists(DATA), reason="dataset not extracted")
def test_real_formations_are_reachable_under_the_mask():
    """On real data: every teacher-forced trajectory stays legal and ends in the human formation."""
    with open(DATA, encoding="utf-8") as f:
        records = [json.loads(next(f)) for _ in range(400)]
    vocab = build_vocab(records)
    rng = np.random.default_rng(0)
    kept = 0
    for r in records:
        proto, why = build_proto(r, vocab)
        if proto is None:
            continue
        kept += 1
        for flip in (False, True):
            ep = Episode(proto, flip)
            for o in teacher_steps(proto, rng, flip):
                assert (o["target"] & ~o["legal"]).sum() == 0
            # replay one trajectory to its end and compare
            while not ep.done:
                ep.step(int(rng.choice(np.flatnonzero(ep.consistent_mask() & ep.legal_mask()))))
            assert ep.formation() == human_formation(proto)
    assert kept > 350
