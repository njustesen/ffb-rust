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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Asserts that {@link BallMoves#passWeight} reproduces Rust's {@code pass_weight} exactly.
 *
 * <p>The engine's grading of the six die faces is fed IN by the golden rather than recomputed here.
 * That is deliberate and consistent with the other fixtures: the pass tables are shared engine
 * mechanics that the parity matrix already covers, so re-deriving them in the test would pin a
 * second copy instead of the arithmetic on top. {@link BallMoves#gradeFaces} is the production path
 * that calls the real mechanic, and the live gate is what exercises it.
 *
 * <p>The fixture asserts its own coverage: an illegal throw (Blizzard rules Long out entirely) must
 * appear, and the accurate/fumble split must vary — a fixture where every throw grades 2/2 would
 * pass while testing one point of a three-outcome model.
 */
class PassWeightTest {

    private static final String PROPERTY = "ffb.passGolden";
    private static final String RELATIVE = "crates/ffb-engine/src/agent/testdata/pass_golden.txt";

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
            "pass golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    private static final class Row {
        boolean home;
        int nr;
        int x;
        int y;
        boolean standing;
        int ma;
        int ag;
        int st;
    }

    @Test
    void passWeightMatchesRust() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);

        String caseName = null;
        boolean blizzard = false;
        int turnNr = 0;
        int dNow = 0;
        int turnsLeft = 0;
        float unact = 1.0f;
        List<Row> rows = new ArrayList<>();
        Map<Integer, Row> homeByNr = new HashMap<>();
        FieldCoordinate ball = null;
        Features f = null;
        BallMoves.Ctx ctx = null;
        ValueModel.Mover m = null;
        int checked = 0;
        int illegalSeen = 0;
        java.util.Set<String> splits = new java.util.HashSet<>();

        for (String raw : lines) {
            String line = raw.trim();
            if (line.isEmpty() || line.startsWith("#")) {
                continue;
            }
            String[] p = line.split("\\s+");
            switch (p[0]) {
                case "case":
                    caseName = p[1];
                    blizzard = "Blizzard".equals(p[3]);
                    turnNr = Integer.parseInt(p[4]);
                    dNow = Integer.parseInt(p[5]);
                    turnsLeft = Integer.parseInt(p[6]);
                    unact = Float.intBitsToFloat((int) Long.parseLong(p[7], 16));
                    rows = new ArrayList<>();
                    homeByNr = new HashMap<>();
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
                    r.ma = Integer.parseInt(p[6]);
                    r.ag = Integer.parseInt(p[7]);
                    r.st = Integer.parseInt(p[8]);
                    rows.add(r);
                    if (r.home) {
                        homeByNr.put(r.nr, r);
                    }
                    break;
                }
                case "ball":
                    ball = new FieldCoordinate(Integer.parseInt(p[1]), Integer.parseInt(p[2]));
                    break;
                case "pass": {
                    if (f == null) {
                        List<Features.Snap> snaps = new ArrayList<>();
                        for (Row r : rows) {
                            snaps.add(new Features.Snap(r.home, r.x, r.y, r.standing, true,
                                r.ma, r.st, r.nr));
                        }
                        f = Features.build(snaps,
                            new Features.BoardState(ball, true, false, false, false), true);
                        ctx = new BallMoves.Ctx(turnNr, blizzard);
                        m = new ValueModel.Mover(true, true, 6, 3, 3, false, false, false,
                            dNow, turnsLeft, unact);
                    }
                    Row rr = homeByNr.get(Integer.parseInt(p[1]));
                    FieldCoordinate from =
                        new FieldCoordinate(Integer.parseInt(p[2]), Integer.parseInt(p[3]));
                    String dist = p[4];
                    int nAcc = Integer.parseInt(p[5]);
                    int nFum = Integer.parseInt(p[6]);
                    String where = caseName + "/home_" + p[1] + " from " + from;

                    if ("NONE".equals(dist)) {
                        // Not a legal throw: Rust returns None and the option must not exist.
                        assertEquals("-", p[8],
                            "an illegal distance must carry no weight (" + where + ")");
                        illegalSeen++;
                        break;
                    }
                    splits.add(nAcc + "/" + nFum);
                    BallMoves.RcvSpec spec = new BallMoves.RcvSpec(
                        new FieldCoordinate(rr.x, rr.y), rr.ma, rr.ag, rr.st, true,
                        false, false, false);
                    float got = BallMoves.passWeight(f, ctx, spec, from, m, nAcc, nFum);
                    assertEquals((int) Long.parseLong(p[8], 16), Float.floatToRawIntBits(got),
                        "passWeight " + where);
                    checked++;
                    break;
                }
                default:
                    break;
            }
        }
        assertTrue(checked >= 8, "too few passes checked: " + checked);
        assertTrue(illegalSeen > 0,
            "no illegal throw in the fixture; the None branch is untested");
        assertTrue(splits.size() >= 3,
            "the accurate/fumble split never varied (" + splits + "); a three-outcome model "
                + "tested at one point");
    }
}
