package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Asserts that {@link Activation} reproduces Rust's tier-1 ranking exactly — the same
 * {@code wPlayer} for every player, and the same ranked ORDER.
 *
 * <p>The order is checked separately from the weights because it is the part with consequences the
 * weights do not have: only the top {@code TIER2} players get a search, so a reordering silently
 * changes which players are properly considered rather than which move is chosen.
 *
 * <p>The test asserts that the fixture reaches at least five distinct rungs of the ladder. It is a
 * six-way else-if chain, and a fixture that only ever lands on the bottom two would pass while
 * testing one comparison.
 */
class ActivationTest {

    private static final String PROPERTY = "ffb.activateGolden";
    private static final String RELATIVE =
        "crates/ffb-engine/src/agent/testdata/activate_golden.txt";

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
            "activate golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    private static final class Row {
        boolean home;
        int nr;
        int x;
        int y;
        boolean standing;
        boolean negatrait;
    }

    @Test
    void tierOneRankingMatchesRust() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);

        String caseName = null;
        int awaiting = 0;
        List<Row> rows = new ArrayList<>();
        FieldCoordinate ball = null;
        boolean loose = false;
        Features f = null;
        List<Activation.Cand> cands = new ArrayList<>();
        int checked = 0;
        Set<Integer> rungs = new HashSet<>();

        for (String raw : lines) {
            String line = raw.trim();
            if (line.isEmpty() || line.startsWith("#")) {
                continue;
            }
            String[] p = line.split("\\s+");
            switch (p[0]) {
                case "case":
                    caseName = p[1];
                    awaiting = Integer.parseInt(p[3]);
                    rows = new ArrayList<>();
                    ball = null;
                    loose = false;
                    f = null;
                    cands = new ArrayList<>();
                    break;
                case "player": {
                    Row r = new Row();
                    r.home = "home".equals(p[1]);
                    r.nr = Integer.parseInt(p[2]);
                    r.x = Integer.parseInt(p[3]);
                    r.y = Integer.parseInt(p[4]);
                    r.standing = "standing".equals(p[5]);
                    r.negatrait = Boolean.parseBoolean(p[6]);
                    rows.add(r);
                    break;
                }
                case "ball":
                    ball = new FieldCoordinate(Integer.parseInt(p[1]), Integer.parseInt(p[2]));
                    loose = Boolean.parseBoolean(p[3]);
                    break;
                case "cand": {
                    if (f == null) {
                        List<Features.Snap> snaps = new ArrayList<>();
                        for (Row r : rows) {
                            snaps.add(new Features.Snap(r.home, r.x, r.y, r.standing, true,
                                6, 3, r.nr));
                        }
                        f = Features.build(snaps,
                            new Features.BoardState(ball, ball != null, loose, false, false), true);
                    }
                    int nr = Integer.parseInt(p[1]);
                    Row me = null;
                    for (Row r : rows) {
                        if (r.home && r.nr == nr) {
                            me = r;
                            break;
                        }
                    }
                    FieldCoordinate at = new FieldCoordinate(me.x, me.y);
                    int i = Features.ix(me.x, me.y);
                    boolean isCarrier = f.ballCarried && f.ball != null && f.ball.equals(at);
                    boolean marked = (f.tz[Features.sideIdx(true)][i] & 0xff) > 0;
                    ValueModel.Mover m = new ValueModel.Mover(true, isCarrier, 6, 3, 3,
                        false, false, false, ValueModel.endzoneDistance(me.x, true),
                        Math.max(8 - Integer.parseInt("3"), 0), f.unactivated[0]);
                    float proxy = Plans.proxyValue(f, at, m);
                    float w = Activation.playerWeight(isCarrier, marked,
                        Activation.canFetch(f, at, m.ma), !me.standing, proxy, me.negatrait,
                        awaiting == nr);
                    String where = caseName + "/home_" + nr;
                    assertEquals((int) Long.parseLong(p[2], 16), Float.floatToRawIntBits(w),
                        "wPlayer " + where);
                    assertEquals((int) Long.parseLong(p[3], 16), Float.floatToRawIntBits(proxy),
                        "proxy " + where);
                    rungs.add(Float.floatToRawIntBits(w));
                    cands.add(new Activation.Cand("home_" + nr, 0, nr, w, proxy));
                    checked++;
                    break;
                }
                case "rank": {
                    String[] want = p[1].split(",");
                    List<Activation.Cand> got = Activation.rank(cands);
                    assertEquals(want.length, got.size(), "rank size (" + caseName + ")");
                    for (int k = 0; k < want.length; k++) {
                        assertEquals(Integer.parseInt(want[k]), got.get(k).nr,
                            "rank position " + k + " (" + caseName + ")");
                    }
                    break;
                }
                default:
                    break;
            }
        }
        assertTrue(checked >= 11, "too few candidates: " + checked);
        assertTrue(rungs.size() >= 5,
            "the ladder reached only " + rungs.size() + " distinct values; it is a six-way chain "
                + "and a fixture landing on two of them tests one comparison");
    }
}
