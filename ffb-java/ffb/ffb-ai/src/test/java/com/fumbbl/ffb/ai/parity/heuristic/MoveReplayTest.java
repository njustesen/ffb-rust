package com.fumbbl.ffb.ai.parity.heuristic;

import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Asserts that {@link MoveReplay#decide} reproduces Rust's {@code replay_plan} on EVERY input.
 *
 * <p>Not a sample. The input space is ten booleans, seven plan kinds and seven relevant player
 * actions, so the golden holds all 50,176 combinations and this walks them all. There is no branch
 * it can miss and no board chosen badly — which matters here more than anywhere else in the
 * campaign, because the state machine has seven exits and nothing about its shape suggests which
 * combinations are the interesting ones.
 *
 * <p>Every other fixture in this campaign samples and relies on a deliberate perturbation to prove
 * the sample was adequate. Three times that check found the sample was NOT adequate. Where
 * exhaustive enumeration is affordable, it removes the question.
 */
class MoveReplayTest {

    private static final String PROPERTY = "ffb.replayGolden";
    private static final String RELATIVE = "crates/ffb-engine/src/agent/testdata/replay_golden.txt";

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
            "replay golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    private static MoveReplay.Kind kindOf(String name) {
        switch (name) {
            case "Move":
                return MoveReplay.Kind.MOVE;
            case "Pickup":
                return MoveReplay.Kind.PICKUP;
            case "Immediate":
                return MoveReplay.Kind.IMMEDIATE;
            case "Blitz":
                return MoveReplay.Kind.BLITZ;
            case "Foul":
                return MoveReplay.Kind.FOUL;
            case "Pass":
                return MoveReplay.Kind.PASS;
            default:
                return MoveReplay.Kind.HAND_OFF;
        }
    }

    private static char verdictChar(MoveReplay.Verdict v) {
        switch (v) {
            case DELIVER_PATH:
                return 'D';
            case FIRE_TERMINAL:
                return 'F';
            case END_PLAYER_ACTION:
                return 'E';
            default:
                return 'R';
        }
    }

    @Test
    void replayMatchesRustOnEveryInput() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);
        int rows = 0;
        int verdicts = 0;
        Set<Character> seen = new HashSet<>();

        for (String raw : lines) {
            String line = raw.trim();
            if (!line.startsWith("row ")) {
                continue;
            }
            String[] p = line.split("\\s+", 4);
            MoveReplay.Kind kind = kindOf(p[1]);
            String paNow = "None".equals(p[2]) ? null : p[2];
            String want = p[3];
            assertEquals(1024, want.length(), "row width for " + p[1] + "/" + p[2]);

            for (int bits = 0; bits < 1024; bits++) {
                MoveReplay.Facts f = new MoveReplay.Facts(paNow,
                    bit(bits, 5), bit(bits, 4), bit(bits, 3), bit(bits, 2), bit(bits, 1),
                    bit(bits, 0));
                char got = verdictChar(MoveReplay.decide(kind, bit(bits, 9), bit(bits, 8),
                    bit(bits, 7), bit(bits, 6), f));
                if (want.charAt(bits) != got) {
                    // Spell the input out; a bit index is unreadable on its own.
                    assertEquals(want.charAt(bits), got, String.format(
                        "%s/%s: isMine=%b pathEmpty=%b delivered=%b fired=%b blocked=%b "
                            + "fouled=%b adjacent=%b onPitch=%b includesNext=%b squaresEmpty=%b",
                        p[1], p[2], bit(bits, 9), bit(bits, 8), bit(bits, 7), bit(bits, 6),
                        bit(bits, 5), bit(bits, 4), bit(bits, 3), bit(bits, 2), bit(bits, 1),
                        bit(bits, 0)));
                }
                seen.add(got);
                verdicts++;
            }
            rows++;
        }
        assertEquals(49, rows, "expected 7 kinds x 7 actions");
        assertEquals(50176, verdicts, "expected every combination");
        assertEquals(4, seen.size(), "all four verdicts must occur somewhere: " + seen);
    }

    private static boolean bit(int bits, int shift) {
        return ((bits >> shift) & 1) == 1;
    }
}
