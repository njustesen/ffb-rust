package com.fumbbl.ffb.ai.parity.heuristic;

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
 * Asserts that {@link Features} reproduces the Rust CORE raster tier exactly.
 *
 * <p>Reads the file the Rust side generates
 * ({@code agent/testdata/features_golden.txt}, via the ignored {@code emit_features_golden} test),
 * so the two cannot drift.
 *
 * <p><b>Why a fixture and not a parity sweep.</b> These three arrays are the foundation the whole
 * value model stands on: the reach search walks them, the threat and support tiers seed off them,
 * and every arrival weight reads them. A disagreement here cannot produce anything but nonsense
 * downstream — and it would surface as a state-hash mismatch hundreds of steps into a game, with
 * nothing near the failure pointing at a raster. Pinning them costs a minute instead.
 */
class FeaturesTest {

    private static final String PROPERTY = "ffb.featuresGolden";
    private static final String RELATIVE =
        "crates/ffb-engine/src/agent/testdata/features_golden.txt";

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
            "features golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    private static byte[] unhex(String hex) {
        byte[] out = new byte[hex.length() / 2];
        for (int i = 0; i < out.length; i++) {
            out[i] = (byte) Integer.parseInt(hex.substring(2 * i, 2 * i + 2), 16);
        }
        return out;
    }

    private static void assertBytes(String what, String board, byte[] want, byte[] got) {
        assertEquals(want.length, got.length, what + " length (board " + board + ")");
        for (int i = 0; i < want.length; i++) {
            if (want[i] != got[i]) {
                int x = i % Features.W;
                int y = i / Features.W;
                assertEquals(want[i] & 0xff, got[i] & 0xff,
                    String.format("%s at (%d,%d) on board %s", what, x, y, board));
            }
        }
    }

    @Test
    void coreRastersMatchRust() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);
        String board = null;
        List<Features.Snap> snaps = new ArrayList<>();
        Features f = null;
        int boardsChecked = 0;
        int arraysChecked = 0;

        for (String raw : lines) {
            String line = raw.trim();
            if (line.isEmpty() || line.startsWith("#")) {
                continue;
            }
            String[] p = line.split("\\s+");
            switch (p[0]) {
                case "board":
                    board = p[1];
                    snaps = new ArrayList<>();
                    f = null;
                    boardsChecked++;
                    break;
                case "player":
                    // The ACTIVE flag comes from the FILE, not from an assumption here. It is a
                    // separate bit from standing/prone, and assuming it was the one thing this
                    // fixture disagreed on when it was first run — the arithmetic was right and
                    // the fixture's own input was wrong.
                    snaps.add(new Features.Snap("home".equals(p[1]), Integer.parseInt(p[3]),
                        Integer.parseInt(p[4]), "standing".equals(p[5]), "active".equals(p[6])));
                    break;
                case "occ":
                    f = Features.build(snaps);
                    assertBytes("occ", board, unhex(p[1]), f.occ);
                    arraysChecked++;
                    break;
                case "tz":
                    assertBytes("tz[" + p[1] + "]", board, unhex(p[2]),
                        f.tz[Integer.parseInt(p[1])]);
                    arraysChecked++;
                    break;
                case "rowprefix": {
                    String[] want = p[2].split(",");
                    int[] got = f.rowPrefix[Integer.parseInt(p[1])];
                    assertEquals(want.length, got.length, "rowPrefix length (board " + board + ")");
                    for (int i = 0; i < want.length; i++) {
                        assertEquals(Integer.parseInt(want[i]), got[i],
                            String.format("rowPrefix[%s] entry %d (row %d, col %d) on board %s",
                                p[1], i, i / (Features.W + 1), i % (Features.W + 1), board));
                    }
                    arraysChecked++;
                    break;
                }
                case "unact":
                    assertEquals(Integer.parseInt(p[2], 16),
                        Float.floatToRawIntBits(f.unactivated[Integer.parseInt(p[1])]),
                        "unactivated[" + p[1] + "] on board " + board);
                    arraysChecked++;
                    break;
                default:
                    break;
            }
        }
        assertTrue(boardsChecked >= 5, "too few boards: " + boardsChecked);
        assertTrue(arraysChecked >= 35, "too few arrays: " + arraysChecked);
    }
}
