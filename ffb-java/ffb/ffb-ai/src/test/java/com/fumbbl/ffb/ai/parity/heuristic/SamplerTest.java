package com.fumbbl.ffb.ai.parity.heuristic;

import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Asserts that {@link Sampler} reproduces the Rust sampler exactly — the same {@code unit()}
 * sequence, the same picked index, and the same number of draws consumed.
 *
 * <p>Reads the same file the Rust side generates
 * ({@code agent/testdata/sampler_golden.txt}, via the ignored {@code emit_sampler_golden} test),
 * so the two cannot drift.
 *
 * <p><b>Why the draw count is checked and not just the index.</b> Two implementations can agree on
 * the answer while disagreeing on the cost: if one spends an extra draw, the next decision reads a
 * different number and everything after it is unrelated. That shows up as a state-hash mismatch
 * hundreds of steps later, with nothing near the failure pointing at the sampler.
 */
class SamplerTest {

    private static final String PROPERTY = "ffb.samplerGolden";
    private static final String RELATIVE = "crates/ffb-engine/src/agent/testdata/sampler_golden.txt";

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
            "sampler golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    private static float f(String hex) {
        return Float.intBitsToFloat((int) Long.parseLong(hex, 16));
    }

    /** The raw draw sequence. If this diverges, nothing downstream can agree. */
    @Test
    void unitSequenceMatchesRust() throws IOException {
        int checked = 0;
        long currentSeed = Long.MIN_VALUE;
        Sampler s = null;
        for (String raw : Files.readAllLines(goldenPath(), StandardCharsets.UTF_8)) {
            String line = raw.trim();
            if (!line.startsWith("unit ")) {
                continue;
            }
            String[] p = line.split("\\s+");
            long seed = Long.parseLong(p[1]);
            int i = Integer.parseInt(p[2]);
            if (seed != currentSeed) {
                currentSeed = seed;
                s = new Sampler(seed, 1.0f);
            }
            int got = Float.floatToRawIntBits(s.unit());
            int want = (int) Long.parseLong(p[3], 16);
            assertEquals(want, got,
                String.format("unit(seed=%d, i=%d): got %s, Rust %s",
                    seed, i, Float.intBitsToFloat(got), Float.intBitsToFloat(want)));
            checked++;
        }
        assertTrue(checked >= 100, "too few unit vectors: " + checked);
    }

    /** Whole decisions: the picked index AND the number of draws it cost. */
    @Test
    void pickMatchesRust() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);
        int checked = 0;
        // The golden file replays each case 40 times against ONE sampler, so the eps escape is
        // actually exercised. Reset only when the (seed, scale, tbase, weights) case changes.
        String currentCase = null;
        Sampler s = null;
        for (String raw : lines) {
            String line = raw.trim();
            if (!line.startsWith("pick ")) {
                continue;
            }
            String[] p = line.split("\\s+");
            long seed = Long.parseLong(p[1]);
            float scale = f(p[2]);
            float tbase = f(p[3]);
            int n = Integer.parseInt(p[4]);
            String[] wHex = p[5].split(",");
            int wantIdx = Integer.parseInt(p[6]);
            int wantDraws = Integer.parseInt(p[7]);

            String key = seed + "|" + p[2] + "|" + p[3] + "|" + p[5];
            if (!key.equals(currentCase)) {
                currentCase = key;
                s = new Sampler(seed, scale);
            }

            s.clear();
            for (int i = 0; i < n; i++) {
                s.push(f(wHex[i]));
            }
            long before = s.drawCount();
            int gotIdx = s.pick(tbase);
            long gotDraws = s.drawCount() - before;

            assertEquals(wantIdx, gotIdx,
                String.format("pick index (seed=%d scale=%s tbase=%s n=%d)", seed, scale, tbase, n));
            assertEquals(wantDraws, gotDraws,
                String.format("DRAW COUNT (seed=%d scale=%s tbase=%s n=%d)", seed, scale, tbase, n));
            checked++;
        }
        assertTrue(checked >= 500, "too few pick vectors: " + checked);
    }
}
