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
import static org.junit.jupiter.api.Assertions.fail;

/**
 * Asserts that {@link DetMath} reproduces the Rust implementation bit for bit.
 *
 * <p>This is the single most load-bearing test of the Java heuristic-agent port. Every scored
 * weight flows through {@code expF32}, and the Dijkstra key through {@code lnF32}; if these two
 * functions disagree with Rust by even one ulp, weights diverge, a softmax draw eventually selects
 * a different option, and the two games desynchronise in a way that looks like an engine bug
 * hundreds of steps later.
 *
 * <p>It deliberately reads the <b>same file</b> the Rust test reads rather than a copy, so the two
 * cannot drift apart. Regenerate with the Rust side's ignored test:
 * {@code cargo test -p ffb-engine --lib det_math::tests::emit_golden_table -- --ignored}.
 */
class DetMathTest {

    /** Override with {@code -Dffb.detMathGolden=<path>} when the repos are laid out differently. */
    private static final String PROPERTY = "ffb.detMathGolden";

    private static final String RELATIVE =
        "crates/ffb-engine/src/agent/testdata/det_math_golden.txt";

    private static Path goldenPath() {
        String override = System.getProperty(PROPERTY);
        if (override != null && !override.isEmpty()) {
            return Paths.get(override);
        }
        // Same shape as the classpath/server-dir resolution in the Rust harness: a candidate list,
        // absolute first, then walk up from wherever the test happens to be running.
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
        Path cwd = Paths.get("").toAbsolutePath();
        // Fail loudly. Skipping would turn the port's most important check into a no-op.
        throw new IllegalStateException(
            "det_math golden table not found (cwd=" + cwd + "). Pass -D" + PROPERTY + "=<path>.");
    }

    @Test
    void matchesTheRustGoldenTable() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);
        int checked = 0;
        for (String raw : lines) {
            String line = raw.trim();
            if (line.isEmpty() || line.startsWith("#")) {
                continue;
            }
            String[] parts = line.split("\\s+");
            assertEquals(3, parts.length, "malformed golden line: " + line);
            // Bit patterns are unsigned 32-bit, so parse as long and narrow.
            int inBits = (int) Long.parseLong(parts[1], 16);
            int outBits = (int) Long.parseLong(parts[2], 16);
            float in = Float.intBitsToFloat(inBits);
            int got;
            switch (parts[0]) {
                case "exp":
                    got = Float.floatToRawIntBits(DetMath.expF32(in));
                    break;
                case "ln":
                    got = Float.floatToRawIntBits(DetMath.lnF32(in));
                    break;
                default:
                    fail("unknown fn " + parts[0]);
                    return;
            }
            assertEquals(
                outBits, got,
                String.format(
                    "%s(%s): got 0x%08x (%s), Rust 0x%08x (%s)",
                    parts[0], in, got, Float.intBitsToFloat(got),
                    outBits, Float.intBitsToFloat(outBits)));
            checked++;
        }
        assertTrue(checked >= 200, "golden table too small: " + checked);
    }

    /** Rust's {@code f32::round} is ties-away-from-zero; neither Math.round nor Math.rint is. */
    @Test
    void roundTiesAwayFromZero() {
        assertEquals(1.0f, DetMath.roundTiesAway(0.5f));
        assertEquals(-1.0f, DetMath.roundTiesAway(-0.5f));
        assertEquals(2.0f, DetMath.roundTiesAway(1.5f));
        assertEquals(-2.0f, DetMath.roundTiesAway(-1.5f));
        assertEquals(2.0f, DetMath.roundTiesAway(2.4f));
        assertEquals(3.0f, DetMath.roundTiesAway(2.5f));
        assertEquals(0.0f, DetMath.roundTiesAway(0.0f));
    }

    @Test
    void edgesAreDefined() {
        assertEquals(Float.floatToRawIntBits(1.0f), Float.floatToRawIntBits(DetMath.expF32(0.0f)));
        assertEquals(0.0f, DetMath.expF32(-1000.0f));
        assertEquals(Float.POSITIVE_INFINITY, DetMath.expF32(1000.0f));
        assertTrue(Float.isNaN(DetMath.expF32(Float.NaN)));
        assertEquals(Float.floatToRawIntBits(0.0f), Float.floatToRawIntBits(DetMath.lnF32(1.0f)));
        assertEquals(Float.NEGATIVE_INFINITY, DetMath.lnF32(0.0f));
        assertTrue(Float.isNaN(DetMath.lnF32(-1.0f)));
        assertTrue(Float.isNaN(DetMath.lnF32(Float.NaN)));
    }
}
