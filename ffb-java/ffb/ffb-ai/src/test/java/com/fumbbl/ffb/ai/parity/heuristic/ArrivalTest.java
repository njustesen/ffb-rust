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
 * Asserts that {@link Arrival} reproduces Rust's {@code arrival_parts} exactly — and that it does so
 * by composing the same {@link Reach} and {@link ValueModel} the fixtures already pin.
 *
 * <p>All four parts are compared, not only the total. {@code w} is a sum of three terms, and three
 * terms reaching the same sum by different routes is exactly the disagreement a single number
 * hides: a value model that is too generous and a turnover cost that is too harsh cancel out on one
 * board and diverge on the next.
 */
class ArrivalTest {

    private static final String PROPERTY = "ffb.arrivalGolden";
    private static final String RELATIVE = "crates/ffb-engine/src/agent/testdata/arrival_golden.txt";

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
            "arrival golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    private static void assertFloats(String what, String where, String hex, float[] got) {
        assertEquals(hex.length() / 8, got.length, what + " length (" + where + ")");
        for (int i = 0; i < got.length; i++) {
            int want = (int) Long.parseLong(hex.substring(8 * i, 8 * i + 8), 16);
            int have = Float.floatToRawIntBits(got[i]);
            if (want != have) {
                assertEquals(Float.intBitsToFloat(want), Float.intBitsToFloat(have),
                    String.format("%s at (%d,%d) for %s",
                        what, i % Features.W, i / Features.W, where));
                assertEquals(want, have, String.format("%s BITS at (%d,%d) for %s",
                    what, i % Features.W, i / Features.W, where));
            }
        }
    }

    @Test
    void arrivalPartsMatchRust() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);

        String caseName = null;
        String where = null;
        List<Features.Snap> snaps = new ArrayList<>();
        FieldCoordinate ball = null;
        boolean loose = false;
        Arrival[] parts = null;
        int moversChecked = 0;
        boolean sawGfi = false;
        boolean sawTouchdownShortCircuit = false;

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
                    boolean sureHands = Boolean.parseBoolean(p[6]);
                    boolean sideStep = Boolean.parseBoolean(p[7]);
                    boolean hasCatch = Boolean.parseBoolean(p[8]);
                    int dNow = Integer.parseInt(p[9]);
                    int turnsLeft = Integer.parseInt(p[10]);
                    float unact = Float.intBitsToFloat((int) Long.parseLong(p[11], 16));
                    boolean teamRr = Boolean.parseBoolean(p[12]);

                    // The mover's own stats have to be on his Snap too, or reach and value would be
                    // describing different players.
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

                    Features f = Features.build(board,
                        new Features.BoardState(ball, ball != null, loose, false, false), true);
                    Reach.Budget b = Reach.budgetOf(
                        new FieldCoordinate(mv.x, mv.y), ma, false, 0);
                    Reach r = Reach.search(f, b, new Reach.MoverSpec(true, ag, false, false),
                        false, false, teamRr);
                    assertNotNull(r, "search returned null for " + where);
                    ValueModel.Mover m = new ValueModel.Mover(true, isCarrier, ma, ag, str,
                        sureHands, sideStep, hasCatch, dNow, turnsLeft, unact);
                    parts = new Arrival[Features.CELLS];
                    for (int i = 0; i < Features.CELLS; i++) {
                        parts[i] = Arrival.parts(f, r, i, m);
                        if (parts[i].gfi > 0) {
                            sawGfi = true;
                        }
                        if (isCarrier && ValueModel.endzoneDistance(i % Features.W, true) == 0
                                && parts[i].v == 1.0f) {
                            sawTouchdownShortCircuit = true;
                        }
                    }
                    moversChecked++;
                    break;
                }
                case "w": {
                    float[] got = new float[Features.CELLS];
                    for (int i = 0; i < got.length; i++) {
                        got[i] = parts[i].w;
                    }
                    assertFloats("w", where, p[1], got);
                    break;
                }
                case "parrive": {
                    float[] got = new float[Features.CELLS];
                    for (int i = 0; i < got.length; i++) {
                        got[i] = parts[i].pArrive;
                    }
                    assertFloats("pArrive", where, p[1], got);
                    break;
                }
                case "v": {
                    float[] got = new float[Features.CELLS];
                    for (int i = 0; i < got.length; i++) {
                        got[i] = parts[i].v;
                    }
                    assertFloats("v", where, p[1], got);
                    break;
                }
                case "gfi": {
                    String[] want = p[1].split(",");
                    assertEquals(Features.CELLS, want.length, "gfi length (" + where + ")");
                    for (int i = 0; i < want.length; i++) {
                        assertEquals(Integer.parseInt(want[i]), parts[i].gfi,
                            String.format("gfi at (%d,%d) for %s",
                                i % Features.W, i / Features.W, where));
                    }
                    break;
                }
                default:
                    break;
            }
        }
        assertTrue(moversChecked >= 7, "too few movers: " + moversChecked);
        assertTrue(sawGfi, "no square in the fixture needed a rush; the penalties are untested");
        assertTrue(sawTouchdownShortCircuit,
            "the endzone short-circuit never fired; put a carrier within reach of the endzone");
    }
}
