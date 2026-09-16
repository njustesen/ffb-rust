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
 * Asserts that {@link SetupPlacement} reproduces Rust's {@code setup_heuristic} bit for bit: the
 * five role scores per player, the team strength, and — for six boards spanning both roles, both
 * sides, the LOS phase and the open phase — every enumerated option's index, square and weight
 * in enumeration order.
 *
 * <p>Reads the file Rust generates ({@code agent/testdata/setup_golden.txt}, via the ignored
 * {@code emit_setup_golden} test), so the two cannot drift. A weight that differs in one bit
 * would move a softmax boundary and eventually pick a different square on one side; the option
 * ORDER is checked because the sampler returns an index.
 */
class SetupPlacementTest {

    private static final String PROPERTY = "ffb.setupGolden";
    private static final String RELATIVE = "crates/ffb-engine/src/agent/testdata/setup_golden.txt";

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
            "setup golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    private static int bits(String hex) {
        return (int) Long.parseLong(hex, 16);
    }

    private static void assertBits(String what, int want, float got) {
        int have = Float.floatToRawIntBits(got);
        if (want != have) {
            assertEquals(Float.intBitsToFloat(want), got, what + " (Rust " + Integer.toHexString(want)
                + ", Java " + Integer.toHexString(have) + ")");
        }
    }

    /** The golden's `player` line: flags in Rust struct order. */
    private static SetupPlacement.SPlayer parsePlayer(String[] f) {
        SetupPlacement.SPlayer p = new SetupPlacement.SPlayer();
        p.nr = Integer.parseInt(f[1]);
        p.id = "p" + p.nr;
        p.st = Integer.parseInt(f[2]);
        p.av = Integer.parseInt(f[3]);
        p.ma = Integer.parseInt(f[4]);
        p.agg = Integer.parseInt(f[5]);
        p.costK = Integer.parseInt(f[6]);
        String fl = f[7];
        assertEquals(24, fl.length(), "flag count");
        p.block = fl.charAt(0) == '1';
        p.guard = fl.charAt(1) == '1';
        p.standFirm = fl.charAt(2) == '1';
        p.sideStep = fl.charAt(3) == '1';
        p.fend = fl.charAt(4) == '1';
        p.thickSkull = fl.charAt(5) == '1';
        p.mighty = fl.charAt(6) == '1';
        p.frenzy = fl.charAt(7) == '1';
        p.stunty = fl.charAt(8) == '1';
        p.sureHands = fl.charAt(9) == '1';
        p.passer = fl.charAt(10) == '1';
        p.catcher = fl.charAt(11) == '1';
        p.nerves = fl.charAt(12) == '1';
        p.dodge = fl.charAt(13) == '1';
        p.sprint = fl.charAt(14) == '1';
        p.leap = fl.charAt(15) == '1';
        p.bigHand = fl.charAt(16) == '1';
        p.kick = fl.charAt(17) == '1';
        p.kor = fl.charAt(18) == '1';
        p.negatrait = fl.charAt(19) == '1';
        p.noHands = fl.charAt(20) == '1';
        p.loner = fl.charAt(21) == '1';
        p.ballAndChain = fl.charAt(22) == '1';
        p.mustField = fl.charAt(23) == '1';
        return p;
    }

    private static SetupPlacement.SPlayer byNr(List<SetupPlacement.SPlayer> ps, int nr) {
        for (SetupPlacement.SPlayer p : ps) {
            if (p.nr == nr) {
                return p;
            }
        }
        throw new IllegalStateException("no player " + nr);
    }

    @Test
    void scoresAndOptionsMatchRust() throws IOException {
        List<SetupPlacement.SPlayer> players = new ArrayList<>();
        SetupPlacement.Board board = null;
        String fixture = null;
        int first = 0;
        List<int[]> expected = new ArrayList<>();
        int fixtures = 0;
        int roles = 0;

        for (String raw : Files.readAllLines(goldenPath(), StandardCharsets.UTF_8)) {
            String line = raw.trim();
            if (line.isEmpty() || line.startsWith("#")) {
                continue;
            }
            String[] f = line.split("\\s+");
            switch (f[0]) {
                case "player":
                    players.add(parsePlayer(f));
                    break;
                case "roles": {
                    SetupPlacement.SPlayer p = byNr(players, Integer.parseInt(f[1]));
                    assertBits("line nr" + p.nr, bits(f[2]), SetupPlacement.lineScore(p));
                    assertBits("handler nr" + p.nr, bits(f[3]), SetupPlacement.handlerScore(p));
                    assertBits("receiver nr" + p.nr, bits(f[4]), SetupPlacement.receiverScore(p));
                    assertBits("fragile nr" + p.nr, bits(f[5]), SetupPlacement.fragile(p));
                    assertBits("power nr" + p.nr, bits(f[6]), SetupPlacement.power(p));
                    roles++;
                    break;
                }
                case "teampower":
                    assertBits("teampower", bits(f[1]), SetupPlacement.teamPower(players));
                    assertBits("meanma", bits(f[3]), SetupPlacement.meanMa(players));
                    break;
                case "fixture":
                    flush(board, fixture, players, first, expected);
                    if (board != null) {
                        fixtures++;
                    }
                    fixture = f[1];
                    board = new SetupPlacement.Board(Boolean.parseBoolean(f[2]), Boolean.parseBoolean(f[3]));
                    board.weaker = Boolean.parseBoolean(f[4]);
                    board.stronger = Boolean.parseBoolean(f[5]);
                    board.fast = Boolean.parseBoolean(f[6]);
                    board.oppFrenzy = Boolean.parseBoolean(f[7]);
                    first = Integer.parseInt(f[8]);
                    expected.clear();
                    break;
                case "mine":
                    assertNotNull(board);
                    board.placeMine(Integer.parseInt(f[1]), Integer.parseInt(f[2]));
                    break;
                case "opp":
                    assertNotNull(board);
                    board.addOpponent(new FieldCoordinate(Integer.parseInt(f[1]), Integer.parseInt(f[2])));
                    break;
                case "options":
                    break;
                case "opt":
                    expected.add(new int[] {
                        Integer.parseInt(f[1]), Integer.parseInt(f[2]), Integer.parseInt(f[3]), bits(f[4])});
                    break;
                default:
                    throw new IllegalStateException("unknown golden line: " + line);
            }
        }
        flush(board, fixture, players, first, expected);
        if (board != null) {
            fixtures++;
        }
        assertEquals(12, roles, "role lines");
        assertEquals(6, fixtures, "fixtures");
        assertTrue(players.size() == 12);
    }

    private static void flush(SetupPlacement.Board board, String fixture, List<SetupPlacement.SPlayer> players,
        int first, List<int[]> expected) {
        if (board == null) {
            return;
        }
        List<SetupPlacement.Option> got = SetupPlacement.enumerate(board, players.subList(first, players.size()));
        assertEquals(expected.size(), got.size(), fixture + ": option count");
        for (int i = 0; i < got.size(); i++) {
            SetupPlacement.Option o = got.get(i);
            int[] e = expected.get(i);
            assertEquals(e[0], o.player, fixture + " opt " + i + " player");
            assertEquals(e[1], o.x, fixture + " opt " + i + " x");
            assertEquals(e[2], o.y, fixture + " opt " + i + " y");
            assertBits(fixture + " opt " + i + " (" + o.x + "," + o.y + ")", e[3], o.w);
        }
    }
}
