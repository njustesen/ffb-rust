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
 * Asserts that {@link Reach} reproduces the Rust {@code reach_with} exactly — the same key on every
 * square, the same cost, the same GFI count, the same back-pointers, and the same visit set.
 *
 * <p>Reads the file the Rust side generates ({@code agent/testdata/reach_golden.txt}, via the
 * ignored {@code emit_reach_golden} test), so the two cannot drift.
 *
 * <p><b>Why {@code prev} and the paths are checked and not only the keys.</b> Two routes to the same
 * square can carry the identical key — a dodge past one tackle zone costs the same wherever it
 * happens — so an implementation whose heap breaks ties differently produces the same arrival
 * PROBABILITIES by a different ROUTE. The keys would all match and the agent would still walk
 * somewhere else. The back-pointer array is what catches that, and the explicit path walks catch a
 * single wrong pointer in the middle of an otherwise right chain.
 */
class ReachTest {

    private static final String PROPERTY = "ffb.reachGolden";
    private static final String RELATIVE = "crates/ffb-engine/src/agent/testdata/reach_golden.txt";

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
            "reach golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    private static long[] longs(String csv) {
        String[] parts = csv.split(",");
        long[] out = new long[parts.length];
        for (int i = 0; i < parts.length; i++) {
            out[i] = Long.parseLong(parts[i]);
        }
        return out;
    }

    private static void assertCells(String what, String caseName, long[] want, long[] got) {
        assertEquals(want.length, got.length, what + " length (case " + caseName + ")");
        for (int i = 0; i < want.length; i++) {
            if (want[i] != got[i]) {
                assertEquals(want[i], got[i], String.format("%s at (%d,%d) in case %s",
                    what, i % Features.W, i / Features.W, caseName));
            }
        }
    }

    @Test
    void reachMatchesRust() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);

        String caseName = null;
        boolean bb2016 = false;
        boolean blizzard = false;
        boolean teamReRoll = false;
        List<Features.Snap> snaps = new ArrayList<>();
        Reach.MoverSpec mover = null;
        FieldCoordinate start = null;
        int maBase = 0;
        boolean prone = false;
        Reach r = null;
        int casesChecked = 0;
        int pathsChecked = 0;

        for (String raw : lines) {
            String line = raw.trim();
            if (line.isEmpty() || line.startsWith("#")) {
                continue;
            }
            String[] p = line.split("\\s+");
            switch (p[0]) {
                case "case":
                    caseName = p[1];
                    bb2016 = "Bb2016".equals(p[2]);
                    blizzard = "Blizzard".equals(p[3]);
                    teamReRoll = Boolean.parseBoolean(p[4]);
                    snaps = new ArrayList<>();
                    mover = null;
                    r = null;
                    casesChecked++;
                    break;
                case "mover": {
                    boolean home = "home".equals(p[1]);
                    int nr = Integer.parseInt(p[2]);
                    int x = Integer.parseInt(p[3]);
                    int y = Integer.parseInt(p[4]);
                    boolean standing = "standing".equals(p[5]);
                    int ma = Integer.parseInt(p[6]);
                    int st = Integer.parseInt(p[7]);
                    int ag = Integer.parseInt(p[8]);
                    boolean dodge = Boolean.parseBoolean(p[9]);
                    boolean sureFeet = Boolean.parseBoolean(p[10]);
                    snaps.add(new Features.Snap(home, x, y, standing, true, ma, st, nr));
                    // The first mover is the one the search runs for; the rest are the board.
                    if (mover == null) {
                        mover = new Reach.MoverSpec(home, ag, dodge, sureFeet);
                        start = new FieldCoordinate(x, y);
                        maBase = ma;
                        prone = !standing;
                    }
                    break;
                }
                case "budget": {
                    Reach.Budget b = Reach.budgetOf(start, maBase, prone, 0);
                    assertEquals(Integer.parseInt(p[1]), b.ma, "budget ma (case " + caseName + ")");
                    assertEquals(Integer.parseInt(p[2]), b.spent, "budget spent (" + caseName + ")");
                    assertEquals(Integer.parseInt(p[3]), b.cap, "budget cap (" + caseName + ")");
                    assertEquals((int) Long.parseLong(p[4], 16), Float.floatToRawIntBits(b.gate),
                        "budget gate (case " + caseName + ")");
                    Features f = Features.build(snaps,
                        new Features.BoardState(null, false, false, false, false), true);
                    r = Reach.search(f, b, mover, bb2016, blizzard, teamReRoll);
                    assertNotNull(r, "search returned null for case " + caseName);
                    break;
                }
                case "key":
                    assertCells("key", caseName, longs(p[1]), r.key);
                    break;
                case "cost": {
                    long[] got = new long[r.cost.length];
                    for (int i = 0; i < got.length; i++) {
                        got[i] = r.cost[i];
                    }
                    assertCells("cost", caseName, longs(p[1]), got);
                    break;
                }
                case "gfi": {
                    long[] got = new long[r.gfi.length];
                    for (int i = 0; i < got.length; i++) {
                        got[i] = r.gfi[i];
                    }
                    assertCells("gfi", caseName, longs(p[1]), got);
                    break;
                }
                case "prev": {
                    long[] got = new long[r.prev.length];
                    for (int i = 0; i < got.length; i++) {
                        got[i] = r.prev[i];
                    }
                    assertCells("prev", caseName, longs(p[1]), got);
                    break;
                }
                case "order": {
                    long[] want = longs(p[1]);
                    assertEquals(want.length, r.order.length, "order size (case " + caseName + ")");
                    for (int i = 0; i < want.length; i++) {
                        assertEquals(want[i], r.order[i],
                            "order entry " + i + " (case " + caseName + ")");
                    }
                    break;
                }
                case "path": {
                    int x = Integer.parseInt(p[1]);
                    int y = Integer.parseInt(p[2]);
                    List<FieldCoordinate> got = r.pathTo(Features.ix(x, y));
                    StringBuilder sb = new StringBuilder();
                    for (int i = 0; i < got.size(); i++) {
                        if (i > 0) {
                            sb.append(';');
                        }
                        sb.append(got.get(i).getX()).append(',').append(got.get(i).getY());
                    }
                    assertEquals(p[3], sb.toString(),
                        String.format("path to (%d,%d) in case %s", x, y, caseName));
                    pathsChecked++;
                    break;
                }
                default:
                    break;
            }
        }
        assertTrue(casesChecked >= 5, "too few cases: " + casesChecked);
        assertTrue(pathsChecked >= 40, "too few paths: " + pathsChecked);
    }
}
