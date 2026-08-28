package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Asserts that {@link Plans} reproduces Rust's {@code top_moves}, {@code run_up_squares} and
 * {@code risked} exactly — the same cells, in the same ORDER, with the same weights.
 *
 * <p>Order is the whole point. The agent samples an index into these lists, so an implementation
 * that agrees on every weight and transposes one tied pair picks a different square. The golden
 * stores the full ordered lists rather than a set, and this test reports the first position that
 * differs together with both cell indices.
 */
class PlansTest {

    private static final String PROPERTY = "ffb.plansGolden";
    private static final String RELATIVE = "crates/ffb-engine/src/agent/testdata/plans_golden.txt";

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
            "plans golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    @Test
    void planOrderingsMatchRust() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);

        String caseName = null;
        String where = null;
        List<Features.Snap> snaps = new ArrayList<>();
        FieldCoordinate ball = null;
        boolean loose = false;
        Features f = null;
        Reach r = null;
        ValueModel.Mover m = null;
        FieldCoordinate here = null;
        int moversChecked = 0;
        int tiesSeen = 0;

        for (String raw : lines) {
            String line = raw.trim();
            if (line.isEmpty() || line.startsWith("#")) {
                continue;
            }
            String[] p = line.split("\\s+");
            switch (p[0]) {
                case "case":
                    caseName = p[1];
                    snaps = new ArrayList<>();
                    ball = null;
                    loose = false;
                    break;
                case "player":
                    snaps.add(new Features.Snap("home".equals(p[1]), Integer.parseInt(p[3]),
                        Integer.parseInt(p[4]), true, true, 6, 3, Integer.parseInt(p[2])));
                    break;
                case "ball":
                    ball = new FieldCoordinate(Integer.parseInt(p[1]), Integer.parseInt(p[2]));
                    loose = Boolean.parseBoolean(p[3]);
                    break;
                case "mover": {
                    where = caseName + "/" + p[1];
                    boolean isCarrier = Boolean.parseBoolean(p[2]);
                    int ma = Integer.parseInt(p[3]);
                    int ag = Integer.parseInt(p[4]);
                    int str = Integer.parseInt(p[5]);
                    int dNow = Integer.parseInt(p[6]);
                    int turnsLeft = Integer.parseInt(p[7]);
                    float unact = Float.intBitsToFloat((int) Long.parseLong(p[8], 16));
                    boolean teamRr = Boolean.parseBoolean(p[9]);

                    List<Features.Snap> board = new ArrayList<>();
                    Features.Snap mv = null;
                    for (Features.Snap sn : snaps) {
                        if (sn.home && sn.nr == 1) {
                            mv = new Features.Snap(true, sn.x, sn.y, true, true, ma, str, 1);
                            board.add(mv);
                        } else {
                            board.add(sn);
                        }
                    }
                    assertNotNull(mv, "home_01 must be on the board (" + where + ")");
                    here = new FieldCoordinate(mv.x, mv.y);

                    f = Features.build(board,
                        new Features.BoardState(ball, ball != null, loose, false, false), true);
                    r = Reach.search(f, Reach.budgetOf(here, ma, false, 0),
                        new Reach.MoverSpec(true, ag, false, false), false, false, teamRr);
                    assertNotNull(r, "search returned null for " + where);
                    m = new ValueModel.Mover(true, isCarrier, ma, ag, str, false, false, false,
                        dNow, turnsLeft, unact);
                    moversChecked++;
                    break;
                }
                case "topmoves": {
                    String[] want = p[1].split(",");
                    List<Plans.Dest> got = Plans.topMoves(f, r, m, Integer.MAX_VALUE);
                    assertEquals(want.length, got.size(), "topMoves size (" + where + ")");
                    float lastW = Float.MAX_VALUE;
                    for (int k = 0; k < want.length; k++) {
                        String[] wi = want[k].split(":");
                        int wantIdx = Integer.parseInt(wi[0]);
                        int wantBits = (int) Long.parseLong(wi[1], 16);
                        assertEquals(wantIdx, got.get(k).i, String.format(
                            "topMoves position %d: Rust picked cell %d (%d,%d), Java cell %d (%d,%d) — %s",
                            k, wantIdx, wantIdx % Features.W, wantIdx / Features.W,
                            got.get(k).i, got.get(k).i % Features.W, got.get(k).i / Features.W,
                            where));
                        assertEquals(wantBits, Float.floatToRawIntBits(got.get(k).w),
                            "topMoves weight at position " + k + " (" + where + ")");
                        // Count exact ties, so the fixture can assert it is actually exercising
                        // the tie-break rather than a strictly ordered list.
                        if (k > 0 && got.get(k).w == lastW) {
                            tiesSeen++;
                        }
                        lastW = got.get(k).w;
                    }
                    break;
                }
                case "runup": {
                    String[] want = p[1].split(",");
                    List<Integer> got = Plans.runUpSquares(r, m, here);
                    assertEquals(want.length, got.size(), "runUpSquares size (" + where + ")");
                    for (int k = 0; k < want.length; k++) {
                        assertEquals(Integer.parseInt(want[k]), (int) got.get(k),
                            "runUpSquares position " + k + " (" + where + ")");
                    }
                    // The mover's own square is first, unconditionally.
                    assertEquals(Features.ix(here.getX(), here.getY()), (int) got.get(0),
                        "runUpSquares must start at the mover's square (" + where + ")");
                    break;
                }
                case "risked": {
                    for (String probe : p[1].split(",")) {
                        String[] q = probe.split(":");
                        float w = Float.intBitsToFloat((int) Long.parseLong(q[0], 16));
                        float pa = Float.intBitsToFloat((int) Long.parseLong(q[1], 16));
                        int want = (int) Long.parseLong(q[2], 16);
                        assertEquals(want, Float.floatToRawIntBits(Plans.risked(w, pa, m)),
                            String.format("risked(%s, %s) in %s", w, pa, where));
                    }
                    break;
                }
                default:
                    break;
            }
        }
        assertTrue(moversChecked >= 4, "too few movers: " + moversChecked);
        assertTrue(tiesSeen > 0,
            "no two destinations tied on weight, so the tie-break is untested by this fixture");
    }
}
