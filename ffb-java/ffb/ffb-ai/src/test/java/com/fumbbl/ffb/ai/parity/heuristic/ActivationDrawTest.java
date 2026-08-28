package com.fumbbl.ffb.ai.parity.heuristic;

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
 * Asserts that {@link Activation#groupDeclarations} and {@link Activation#chooseCandidate}
 * reproduce Rust's two-level activation draw exactly — the same groups, the same group weights, the
 * same chosen option, and the same number of draws spent.
 *
 * <p>The draw COUNT is checked because it is the part that desynchronises silently. Two
 * {@code softmaxPick} calls normally spend one draw each, but a level with a single entry spends
 * none — so a grouping that produces one group too many or too few costs a draw, and every decision
 * after it reads a different number. That surfaces as a state-hash mismatch far downstream with
 * nothing near the failure pointing at the grouping.
 *
 * <p>The fixture asserts it exercises all three draw counts (0, 1 and 2), since a set of cases that
 * always spends two would leave the singleton path untested.
 */
class ActivationDrawTest {

    private static final String PROPERTY = "ffb.drawGolden";
    private static final String RELATIVE = "crates/ffb-engine/src/agent/testdata/draw_golden.txt";

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
            "draw golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    @Test
    void twoLevelDrawMatchesRust() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);

        String caseName = null;
        List<Activation.Option> options = new ArrayList<>();
        int checked = 0;
        Set<Long> drawCounts = new HashSet<>();

        for (String raw : lines) {
            String line = raw.trim();
            if (line.isEmpty() || line.startsWith("#")) {
                continue;
            }
            String[] p = line.split("\\s+");
            switch (p[0]) {
                case "case":
                    caseName = p[1];
                    options = new ArrayList<>();
                    break;
                case "cand":
                    options.add(new Activation.Option(p[1], p[2],
                        Float.intBitsToFloat((int) Long.parseLong(p[3], 16))));
                    break;
                case "groups": {
                    // The golden includes the EndTurn group; groupDeclarations does not add it, so
                    // append it here exactly as chooseCandidate does.
                    List<List<Integer>> got = Activation.groupDeclarations(options);
                    List<Integer> end = new ArrayList<>();
                    end.add(options.size());
                    got.add(end);

                    String[] want = p[1].split("\\|");
                    assertEquals(want.length, got.size(), "group count (" + caseName + ")");
                    for (int g = 0; g < want.length; g++) {
                        String[] wi = want[g].split(",");
                        assertEquals(wi.length, got.get(g).size(),
                            "size of group " + g + " (" + caseName + ")");
                        for (int k = 0; k < wi.length; k++) {
                            assertEquals(Integer.parseInt(wi[k]), (int) got.get(g).get(k),
                                "group " + g + " entry " + k + " (" + caseName + ")");
                        }
                    }
                    break;
                }
                case "groupw": {
                    List<List<Integer>> got = Activation.groupDeclarations(options);
                    List<Integer> end = new ArrayList<>();
                    end.add(options.size());
                    got.add(end);
                    String[] want = p[1].split(",");
                    assertEquals(want.length, got.size(), "groupw count (" + caseName + ")");
                    for (int g = 0; g < want.length; g++) {
                        float best = -Float.MAX_VALUE;
                        for (int j : got.get(g)) {
                            float w = j < options.size() ? options.get(j).weight : 0.0f;
                            best = Math.max(best, w);
                        }
                        assertEquals((int) Long.parseLong(want[g], 16),
                            Float.floatToRawIntBits(best),
                            "group weight " + g + " (" + caseName + ")");
                    }
                    break;
                }
                case "draw": {
                    float scale = Float.intBitsToFloat((int) Long.parseLong(p[1], 16));
                    int wantIdx = Integer.parseInt(p[2]);
                    long wantDraws = Long.parseLong(p[3]);
                    Sampler s = new Sampler(9, scale);
                    long before = s.drawCount();
                    int got = Activation.chooseCandidate(s, options);
                    long spent = s.drawCount() - before;
                    assertEquals(wantIdx, got,
                        String.format("chosen option (%s, scale %s)", caseName, scale));
                    assertEquals(wantDraws, spent,
                        String.format("DRAW COUNT (%s, scale %s)", caseName, scale));
                    drawCounts.add(spent);
                    checked++;
                    break;
                }
                default:
                    break;
            }
        }
        assertTrue(checked >= 15, "too few draws checked: " + checked);
        assertTrue(drawCounts.contains(0L) && drawCounts.contains(1L) && drawCounts.contains(2L),
            "draw counts seen were " + drawCounts + "; a fixture that never spends 1 leaves the "
                + "singleton-group path untested");
    }
}
