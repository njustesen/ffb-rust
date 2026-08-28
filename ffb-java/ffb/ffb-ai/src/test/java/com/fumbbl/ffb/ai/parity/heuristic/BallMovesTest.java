package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Asserts that {@link BallMoves} reproduces Rust's {@code receiver_of}, {@code handoff_weight} and
 * {@code foul_weight} exactly.
 *
 * <p>All five parts of the receiver are compared — the catch chance, the value, the
 * {@code scoresNow} flag and both turn counts — because the flag and the counts are what the
 * hand-off price branches on, and a value that happens to match with the wrong flag behind it is a
 * disagreement waiting for the next board.
 *
 * <p>The test also asserts that {@code scoresNow} is seen BOTH ways across the fixture. It is the
 * single most consequential boolean in this file — it is the difference between a give worth 1.0
 * and one worth at most 0.20 — and a fixture where it is always false would pass while testing
 * almost nothing.
 */
class BallMovesTest {

    private static final String PROPERTY = "ffb.ballMovesGolden";
    private static final String RELATIVE =
        "crates/ffb-engine/src/agent/testdata/ballmoves_golden.txt";

    private static Path goldenPath() {
        String override = System.getProperty(PROPERTY);
        if (override != null && !override.isEmpty()) {
            return Paths.get(override);
        }
        Path[] candidates = {
            Paths.get("C:/Users/Admin/niels/ffb-rust/ffb-rust").resolve(RELATIVE),
            Paths.get("../../../ffb-rust/ffb-rust").resolve(RELATIVE),
            Paths.get("../../ffb-rust/ffb-rust").resolve(RELATIVE),
            Paths.get("../ffb-rust/ffb-rust").resolve(RELATIVE),
        };
        for (Path c : candidates) {
            if (Files.isRegularFile(c)) {
                return c;
            }
        }
        throw new IllegalStateException(
            "ballmoves golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    private static float f32(String hex) {
        return Float.intBitsToFloat((int) Long.parseLong(hex, 16));
    }

    /** One player as the golden describes him, keyed by side and jersey number. */
    private static final class Row {
        boolean home;
        int nr;
        int x;
        int y;
        boolean standing;
        boolean active;
        int ma;
        int ag;
        int st;
    }

    @Test
    void ballMovesMatchRust() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);

        String caseName = null;
        int turnNr = 0;
        int dNow = 0;
        int turnsLeft = 0;
        float unact = 1.0f;
        List<Row> rows = new ArrayList<>();
        Map<Integer, Row> homeByNr = new HashMap<>();
        Map<Integer, Row> awayByNr = new HashMap<>();
        FieldCoordinate ball = null;
        Features f = null;
        BallMoves.Ctx ctx = null;
        ValueModel.Mover m = null;
        int checked = 0;
        boolean sawScoresNow = false;
        boolean sawNotScoresNow = false;

        for (String raw : lines) {
            String line = raw.trim();
            if (line.isEmpty() || line.startsWith("#")) {
                continue;
            }
            String[] p = line.split("\\s+");
            switch (p[0]) {
                case "case":
                    caseName = p[1];
                    turnNr = Integer.parseInt(p[2]);
                    dNow = Integer.parseInt(p[3]);
                    turnsLeft = Integer.parseInt(p[4]);
                    unact = f32(p[5]);
                    rows = new ArrayList<>();
                    homeByNr = new HashMap<>();
                    awayByNr = new HashMap<>();
                    ball = null;
                    f = null;
                    break;
                case "player": {
                    Row r = new Row();
                    r.home = "home".equals(p[1]);
                    r.nr = Integer.parseInt(p[2]);
                    r.x = Integer.parseInt(p[3]);
                    r.y = Integer.parseInt(p[4]);
                    r.standing = "standing".equals(p[5]);
                    r.active = "active".equals(p[6]);
                    r.ma = Integer.parseInt(p[7]);
                    r.ag = Integer.parseInt(p[8]);
                    r.st = Integer.parseInt(p[9]);
                    rows.add(r);
                    (r.home ? homeByNr : awayByNr).put(r.nr, r);
                    break;
                }
                case "ball":
                    ball = new FieldCoordinate(Integer.parseInt(p[1]), Integer.parseInt(p[2]));
                    break;
                case "receiver":
                case "handoff":
                case "foul": {
                    if (f == null) {
                        List<Features.Snap> snaps = new ArrayList<>();
                        for (Row r : rows) {
                            snaps.add(new Features.Snap(r.home, r.x, r.y, r.standing, r.active,
                                r.ma, r.st, r.nr));
                        }
                        f = Features.build(snaps,
                            new Features.BoardState(ball, ball != null, false, false, false), true);
                        ctx = new BallMoves.Ctx(turnNr, false);
                        m = new ValueModel.Mover(true, true, 6, 3, 3, false, false, false,
                            dNow, turnsLeft, unact);
                    }
                    if ("foul".equals(p[0])) {
                        Row d = awayByNr.get(Integer.parseInt(p[1]));
                        float got = BallMoves.foulWeight(f, Integer.parseInt(p[2]),
                            Integer.parseInt(p[3]), Integer.parseInt(p[4]),
                            new FieldCoordinate(d.x, d.y), 0, m);
                        assertEquals((int) Long.parseLong(p[5], 16), Float.floatToRawIntBits(got),
                            "foulWeight on away_" + p[1] + " (" + caseName + ")");
                        checked++;
                        break;
                    }
                    Row rr = homeByNr.get(Integer.parseInt(p[1]));
                    FieldCoordinate from =
                        new FieldCoordinate(Integer.parseInt(p[2]), Integer.parseInt(p[3]));
                    BallMoves.RcvSpec spec = new BallMoves.RcvSpec(
                        new FieldCoordinate(rr.x, rr.y), rr.ma, rr.ag, rr.st, rr.active,
                        false, false, false);
                    String where = caseName + "/home_" + p[1] + " from " + from;
                    if ("receiver".equals(p[0])) {
                        String[] q = p[4].split(":");
                        BallMoves.Receiver got = BallMoves.receiverOf(f, ctx, spec, from, m);
                        assertEquals((int) Long.parseLong(q[0], 16),
                            Float.floatToRawIntBits(got.pCatch), "pCatch " + where);
                        assertEquals((int) Long.parseLong(q[1], 16),
                            Float.floatToRawIntBits(got.v), "v " + where);
                        assertEquals(Boolean.parseBoolean(q[2]), got.scoresNow,
                            "scoresNow " + where);
                        assertEquals(Integer.parseInt(q[3]), got.tts, "tts " + where);
                        assertEquals(Integer.parseInt(q[4]), got.turns, "turns " + where);
                        if (got.scoresNow) {
                            sawScoresNow = true;
                        } else {
                            sawNotScoresNow = true;
                        }
                    } else {
                        // Rust returns None only when the receiver is off the pitch, which these
                        // boards never produce; a '-' here would mean the fixture changed shape.
                        assertTrue(!"-".equals(p[4]),
                            "golden says handoff was None for " + where + "; unexpected");
                        assertEquals((int) Long.parseLong(p[4], 16),
                            Float.floatToRawIntBits(BallMoves.handoffWeight(f, ctx, spec, from, m)),
                            "handoffWeight " + where);
                    }
                    checked++;
                    break;
                }
                default:
                    break;
            }
        }
        assertTrue(checked >= 30, "too few rows checked: " + checked);
        assertTrue(sawScoresNow && sawNotScoresNow,
            "scoresNow was always " + sawScoresNow + "; the branch that makes a give worth 1.0 "
                + "instead of 0.20 is untested");
    }

    /**
     * Regression guard for ITER45: {@code ActivationDriver.foes} scored every foul target at a
     * hardcoded {@code 0.0f} instead of calling {@link BallMoves#foulWeight}. The arithmetic was
     * ported and golden-tested all along; only the call was missing, so no fixture could see it.
     *
     * <p>This pins the fact that made the placeholder wrong: over the whole plausible input range
     * -- every armour value, both assist directions, all three ball-proximity tiers -- the weight is
     * never zero. A zero foul weight is therefore not a value the model can produce, and any future
     * caller that reports one is not calling this function.
     */
    @Test
    void foulWeightIsNeverZero() {
        List<Features.Snap> snaps = new ArrayList<>();
        snaps.add(new Features.Snap(true, 12, 7, true, true, 6, 3, 1));   // the fouler
        snaps.add(new Features.Snap(false, 13, 7, false, true, 6, 3, 1)); // the victim, down
        FieldCoordinate victim = new FieldCoordinate(13, 7);

        // The three ball-proximity tiers foulWeight distinguishes: carried by the victim, loose
        // next to him, and nowhere near.
        FieldCoordinate[] balls = {victim, new FieldCoordinate(13, 8), new FieldCoordinate(2, 2)};
        boolean[] carried = {true, false, false};

        int cases = 0;
        for (int b = 0; b < balls.length; b++) {
            Features f = Features.build(snaps,
                new Features.BoardState(balls[b], carried[b], false, false, false), true);
            for (int av = 6; av <= 11; av++) {
                for (int off = 0; off <= 3; off++) {
                    for (int dfn = 0; dfn <= 3; dfn++) {
                        for (int bribes = 0; bribes <= 1; bribes++) {
                            for (float unact : new float[] {0.0f, 1.0f}) {
                                ValueModel.Mover m = new ValueModel.Mover(true, true, 6, 3, 3,
                                    false, false, false, 10, 8, unact);
                                float w = BallMoves.foulWeight(f, av, off, dfn, victim, bribes, m);
                                assertNotEquals(0.0f, w, String.format(
                                    "av=%d off=%d dfn=%d bribes=%d unact=%.1f ballTier=%d",
                                    av, off, dfn, bribes, unact, b));
                                cases++;
                            }
                        }
                    }
                }
            }
        }
        assertEquals(3 * 6 * 4 * 4 * 2 * 2, cases);
    }
}
