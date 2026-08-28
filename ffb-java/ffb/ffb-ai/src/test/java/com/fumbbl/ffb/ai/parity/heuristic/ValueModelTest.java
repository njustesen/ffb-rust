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
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Asserts that {@link ValueModel} reproduces Rust's {@code value_at} exactly, for every square and
 * every mover in {@code value_golden.txt}.
 *
 * <p><b>The rule column is checked as well as the value.</b> Two implementations can agree on a
 * number while disagreeing about which branch produced it — a support value that happens to equal a
 * pickup value, say — and that disagreement is invisible until the board changes and the branches
 * diverge. Pinning the branch makes the agreement mean something.
 */
class ValueModelTest {

    private static final String PROPERTY = "ffb.valueGolden";
    private static final String RELATIVE = "crates/ffb-engine/src/agent/testdata/value_golden.txt";

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
            "value golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    private static char ruleChar(ValueModel.Rule r) {
        switch (r) {
            case SCORE_TOUCHDOWN:
                return 'T';
            case SCORE_ADVANCE:
                return 'A';
            case PICKUP:
                return 'P';
            default:
                return 'S';
        }
    }

    @Test
    void valueAtMatchesRust() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);

        String caseName = null;
        String moverName = null;
        List<Features.Snap> snaps = new ArrayList<>();
        FieldCoordinate ball = null;
        boolean loose = false;
        Features f = null;
        ValueModel.Mover mover = null;
        int moversChecked = 0;
        int casesChecked = 0;
        // Every branch has to actually FIRE somewhere, or the fixture proves less than it looks.
        boolean sawTouchdown = false;
        boolean sawAdvance = false;
        boolean sawPickup = false;
        boolean sawSupport = false;

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
                    f = null;
                    casesChecked++;
                    break;
                case "player":
                    snaps.add(new Features.Snap("home".equals(p[1]), Integer.parseInt(p[3]),
                        Integer.parseInt(p[4]), true, true, 6, Integer.parseInt(p[5]),
                        Integer.parseInt(p[2])));
                    break;
                case "ball":
                    ball = new FieldCoordinate(Integer.parseInt(p[1]), Integer.parseInt(p[2]));
                    loose = Boolean.parseBoolean(p[3]);
                    break;
                case "mover":
                    moverName = p[1];
                    f = Features.build(snaps,
                        new Features.BoardState(ball, ball != null, loose, false, false), true);
                    mover = new ValueModel.Mover(
                        Boolean.parseBoolean(p[2]), Boolean.parseBoolean(p[3]),
                        Integer.parseInt(p[4]), Integer.parseInt(p[5]), Integer.parseInt(p[6]),
                        Boolean.parseBoolean(p[7]), Boolean.parseBoolean(p[8]),
                        Boolean.parseBoolean(p[9]), Integer.parseInt(p[10]),
                        Integer.parseInt(p[11]),
                        Float.intBitsToFloat((int) Long.parseLong(p[12], 16)));
                    moversChecked++;
                    break;
                case "value": {
                    String hex = p[1];
                    assertEquals(Features.CELLS, hex.length() / 8,
                        "value length (" + caseName + "/" + moverName + ")");
                    for (int i = 0; i < Features.CELLS; i++) {
                        int want = (int) Long.parseLong(hex.substring(8 * i, 8 * i + 8), 16);
                        int got = Float.floatToRawIntBits(ValueModel.valueAt(f, i, mover).v);
                        if (want != got) {
                            assertEquals(Float.intBitsToFloat(want), Float.intBitsToFloat(got),
                                String.format("value at (%d,%d) for %s/%s",
                                    i % Features.W, i / Features.W, caseName, moverName));
                            assertEquals(want, got,
                                String.format("value BITS at (%d,%d) for %s/%s",
                                    i % Features.W, i / Features.W, caseName, moverName));
                        }
                    }
                    break;
                }
                case "rule": {
                    String want = p[1];
                    assertEquals(Features.CELLS, want.length(),
                        "rule length (" + caseName + "/" + moverName + ")");
                    for (int i = 0; i < Features.CELLS; i++) {
                        char got = ruleChar(ValueModel.valueAt(f, i, mover).rule);
                        assertEquals(want.charAt(i), got,
                            String.format("rule at (%d,%d) for %s/%s",
                                i % Features.W, i / Features.W, caseName, moverName));
                        switch (got) {
                            case 'T': sawTouchdown = true; break;
                            case 'A': sawAdvance = true; break;
                            case 'P': sawPickup = true; break;
                            default: sawSupport = true; break;
                        }
                    }
                    break;
                }
                default:
                    break;
            }
        }
        assertTrue(casesChecked >= 4, "too few cases: " + casesChecked);
        assertTrue(moversChecked >= 14, "too few movers: " + moversChecked);
        assertTrue(sawTouchdown && sawAdvance && sawPickup && sawSupport,
            "not every branch fired: T=" + sawTouchdown + " A=" + sawAdvance
                + " P=" + sawPickup + " S=" + sawSupport);
    }
}
