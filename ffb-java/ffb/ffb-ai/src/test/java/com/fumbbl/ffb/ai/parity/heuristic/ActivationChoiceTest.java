package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashSet;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Asserts that {@link ActivationChoice#choose} reproduces Rust's {@code handle_activate} end to end
 * — the same player, the same declared action, the same block target, at all three temperature
 * scales.
 *
 * <p>This is the fixture the other twelve cannot replace. Each of them pins one layer, and a
 * composition bug is invisible to all of them at once: every layer can be right while the order they
 * run in, or what one hands the next, is wrong. It is also the closest a fixture gets to the live
 * gate — same entry point, same return value, no engine underneath.
 *
 * <p>All three scales are checked because the draw is where they differ, and the fixture asserts
 * that at least one case actually CHANGES its answer between scales. A set of boards where the
 * argmax and the sampled pick always agree would exercise the two-level draw without ever testing
 * it.
 */
class ActivationChoiceTest {

    private static final String PROPERTY = "ffb.actEndGolden";
    private static final String RELATIVE = "crates/ffb-engine/src/agent/testdata/actend_golden.txt";

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
            "actend golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    private static final class Row {
        boolean home;
        int nr;
        int x;
        int y;
        boolean standing;
    }

    @Test
    void activationChoiceMatchesRust() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);

        String caseName = null;
        List<Row> rows = new ArrayList<>();
        FieldCoordinate ball = null;
        boolean loose = false;
        List<int[]> eligNrs = new ArrayList<>();
        List<List<String>> eligActions = new ArrayList<>();
        int checked = 0;
        int casesVaryingByScale = 0;
        String firstAnswerThisCase = null;
        boolean variedThisCase = false;

        for (String raw : lines) {
            String line = raw.trim();
            if (line.isEmpty() || line.startsWith("#")) {
                continue;
            }
            String[] p = line.split("\\s+");
            switch (p[0]) {
                case "case":
                    if (variedThisCase) {
                        casesVaryingByScale++;
                    }
                    caseName = p[1];
                    rows = new ArrayList<>();
                    ball = null;
                    loose = false;
                    eligNrs = new ArrayList<>();
                    eligActions = new ArrayList<>();
                    firstAnswerThisCase = null;
                    variedThisCase = false;
                    break;
                case "player": {
                    Row r = new Row();
                    r.home = "home".equals(p[1]);
                    r.nr = Integer.parseInt(p[2]);
                    r.x = Integer.parseInt(p[3]);
                    r.y = Integer.parseInt(p[4]);
                    r.standing = "standing".equals(p[5]);
                    rows.add(r);
                    break;
                }
                case "ball":
                    ball = new FieldCoordinate(Integer.parseInt(p[1]), Integer.parseInt(p[2]));
                    loose = Boolean.parseBoolean(p[3]);
                    break;
                case "eligible":
                    eligNrs.add(new int[] {Integer.parseInt(p[1])});
                    eligActions.add(Arrays.asList(p[2].split(",")));
                    break;
                case "chose": {
                    float scale = Float.intBitsToFloat((int) Long.parseLong(p[1], 16));
                    String wantPlayer = p[2];
                    String wantAction = p[3];
                    String wantTarget = p[4];

                    final List<Row> board = rows;
                    List<Features.Snap> snaps = new ArrayList<>();
                    for (Row r : board) {
                        snaps.add(new Features.Snap(r.home, r.x, r.y, r.standing, true, 6, 3, r.nr));
                    }
                    Features f = Features.build(snaps,
                        new Features.BoardState(ball, ball != null, loose, false, false), true);

                    List<ActivationChoice.Eligible> elig = new ArrayList<>();
                    for (int i = 0; i < eligNrs.size(); i++) {
                        int nr = eligNrs.get(i)[0];
                        Row me = null;
                        for (Row r : board) {
                            if (r.home && r.nr == nr) {
                                me = r;
                            }
                        }
                        elig.add(new ActivationChoice.Eligible(String.format("home_%02d", nr), nr,
                            new FieldCoordinate(me.x, me.y), me.standing, false,
                            eligActions.get(i), 6, 3, 3, false, false, false, false, false));
                    }

                    ActivationChoice.Board bd = new BoardFromRows(board, f);
                    Sampler s = new Sampler(21, scale);
                    // The golden was emitted with BOTH coverage terms at zero, so this fixture
                    // exercises the sharp arms only: a Coverage built at tempScale 0 returns 0.0
                    // from `novelty` and `floor` whatever the maps hold. The live sweep at
                    // `--heur-scale 1.0` is what covers them.
                    ActivationChoice.Coverage cov = new ActivationChoice.Coverage(
                        0.0f, new java.util.HashMap<>(), new java.util.HashMap<>());
                    ActivationChoice.Decision d = ActivationChoice.choose(f, s, bd, elig, 3, false,
                        null, new HashSet<>(), true, false, false, cov, 0L);

                    String gotPlayer = d.player == null ? "ENDTURN" : d.player;
                    String gotAction = d.player == null ? "-" : d.action;
                    String gotTarget = d.target == null ? "-" : d.target;
                    String where = caseName + " @ scale " + scale;
                    assertEquals(wantPlayer, gotPlayer, "player (" + where + ")");
                    assertEquals(wantAction, gotAction, "action (" + where + ")");
                    assertEquals(wantTarget, gotTarget, "target (" + where + ")");

                    String answer = gotPlayer + "/" + gotAction + "/" + gotTarget;
                    if (firstAnswerThisCase == null) {
                        firstAnswerThisCase = answer;
                    } else if (!firstAnswerThisCase.equals(answer)) {
                        variedThisCase = true;
                    }
                    checked++;
                    break;
                }
                default:
                    break;
            }
        }
        if (variedThisCase) {
            casesVaryingByScale++;
        }
        assertTrue(checked >= 12, "too few decisions checked: " + checked);
        assertTrue(casesVaryingByScale > 0,
            "every case gave the same answer at every scale; the two-level draw is exercised but "
                + "never actually tested");
    }

    /**
     * The decision must not depend on the ORDER the eligible list arrives in.
     *
     * <p>Rust canonically sorts by {@code (side, nr)} before it enumerates anything; the harness
     * supplies ROSTER order. Those agree only while a jersey number equals its roster index, which
     * is true of every lineman team and false of the bb2025 amazons, whose second roster slot wears
     * 13. The draw and the declaration grouping are POSITIONAL, so an unsorted list picks a
     * different candidate out of identical weights -- a divergence with no wrong number anywhere in
     * it.
     *
     * <p>Numbers 1, 13, 3 in roster order are the amazon case exactly: sorted canonically they are
     * 1, 3, 13, so the second and third candidates swap.
     */
    @Test
    void eligibleListOrderDoesNotChangeTheDecision() {
        List<Row> board = new ArrayList<>();
        // Roster order 1, 13, 3, 4, 5, 6 -- the bb2025 amazon shape. Nobody carries the ball and
        // nobody is adjacent to it, so no single candidate dominates and the draw is genuinely
        // spread across players; a fixture with a clear best move answers it the same way whatever
        // order the list is in, and tests nothing.
        int[][] mine = {{1, 11, 4}, {13, 12, 6}, {3, 11, 8}, {4, 12, 10}, {5, 10, 5}, {6, 10, 9}};
        for (int[] m : mine) {
            Row r = new Row();
            r.home = true;
            r.nr = m[0];
            r.x = m[1];
            r.y = m[2];
            r.standing = true;
            board.add(r);
        }
        for (int[] o : new int[][] {{2, 13, 6}, {4, 12, 9}, {7, 20, 7}}) {
            Row r = new Row();
            r.home = false;
            r.nr = o[0];
            r.x = o[1];
            r.y = o[2];
            r.standing = true;
            board.add(r);
        }
        List<Features.Snap> snaps = new ArrayList<>();
        for (Row r : board) {
            snaps.add(new Features.Snap(r.home, r.x, r.y, r.standing, true, 6, 3, r.nr));
        }
        FieldCoordinate ball = new FieldCoordinate(22, 7);
        Features f = Features.build(snaps,
            new Features.BoardState(ball, true, true, false, false), true);
        ActivationChoice.Board bd = new BoardFromRows(board, f);

        List<ActivationChoice.Eligible> roster = new ArrayList<>();
        for (Row r : board) {
            if (!r.home) {
                continue;
            }
            roster.add(new ActivationChoice.Eligible(String.format("home_%02d", r.nr), r.nr,
                new FieldCoordinate(r.x, r.y), true, false,
                Arrays.asList("Move", "Block"), 6, 3, 3, false, false, false, false, false));
        }
        List<ActivationChoice.Eligible> canonical = new ArrayList<>(roster);
        canonical.sort(java.util.Comparator.comparingInt(e -> e.nr));
        assertEquals(13, roster.get(1).nr, "the fixture must actually be out of canonical order");
        assertEquals(3, canonical.get(1).nr);

        for (float scale : new float[] {0.0f, 1.0f, 1.0e6f}) {
            ActivationChoice.Decision a = choose(f, bd, roster, scale);
            ActivationChoice.Decision b = choose(f, bd, canonical, scale);
            assertEquals(describe(b), describe(a), "roster order vs canonical order @ " + scale);
        }
    }

    private static ActivationChoice.Decision choose(Features f, ActivationChoice.Board bd,
            List<ActivationChoice.Eligible> elig, float scale) {
        return ActivationChoice.choose(f, new Sampler(21, scale), bd, elig, 3, false, null,
            new HashSet<>(), true, false, false,
            new ActivationChoice.Coverage(0.0f, new java.util.HashMap<>(),
                new java.util.HashMap<>()),
            0L);
    }

    private static String describe(ActivationChoice.Decision d) {
        if (d.player == null) {
            return "ENDTURN";
        }
        return d.player + "/" + d.action + "/" + d.target + "/" + d.dest;
    }

    /** Supplies the eligibility answers the harness gives in production. */
    private static final class BoardFromRows implements ActivationChoice.Board {
        private final List<Row> rows;
        private final Features f;

        BoardFromRows(List<Row> rows, Features f) {
            this.rows = rows;
            this.f = f;
        }

        private Row find(String id) {
            int nr = Integer.parseInt(id.substring(id.length() - 2));
            for (Row r : rows) {
                if (r.home && r.nr == nr) {
                    return r;
                }
            }
            return null;
        }

        private List<PlanBuilder.BlockTarget> adjacentFoes(String playerId, boolean adjacentOnly) {
            Row me = find(playerId);
            List<PlanBuilder.BlockTarget> out = new ArrayList<>();
            if (me == null) {
                return out;
            }
            ValueModel.Mover m = new ValueModel.Mover(true, false, 6, 3, 3, false, false, false,
                ValueModel.endzoneDistance(me.x, true), 5, f.unactivated[0]);
            for (Row o : rows) {
                if (o.home || !o.standing) {
                    continue;
                }
                int d = Math.max(Math.abs(me.x - o.x), Math.abs(me.y - o.y));
                if (adjacentOnly && d != 1) {
                    continue;
                }
                // block_weight needs the engine's assist arithmetic; on these all-ST3 boards with
                // no assists it reduces to the even-dice rung, which is what the golden encodes.
                out.add(new PlanBuilder.BlockTarget(String.format("away_%02d", o.nr),
                    new FieldCoordinate(o.x, o.y), blockWeight(me, o)));
            }
            return out;
        }

        /** Rust {@code block_weight} for the all-ST3, assist-free case these boards produce. */
        private float blockWeight(Row att, Row def) {
            // Equal strength with no assists is one die: 0.25 without Block.
            float w = 0.25f;
            boolean defHasBall = f.ballCarried && f.ball != null
                && f.ball.getX() == def.x && f.ball.getY() == def.y;
            if (defHasBall) {
                w *= 1.35f;
            }
            return Math.min(Math.max(w, 0.01f), 1.0f);
        }

        @Override
        public List<PlanBuilder.BlockTarget> blockTargets(String playerId) {
            return adjacentFoes(playerId, true);
        }

        @Override
        public List<PlanBuilder.BlockTarget> blitzFoes(String playerId) {
            return adjacentFoes(playerId, false);
        }

        @Override
        public List<PlanBuilder.BlockTarget> foulTargets(String playerId) {
            return new ArrayList<>();
        }

        @Override
        public List<PlanBuilder.Receiver> receivers(String playerId, boolean forPass) {
            return new ArrayList<>();
        }
    }

    /**
     * Regression guard for ITER48: every terminal action name the harness can produce must survive
     * {@link ActivationChoice#moveVariant} as a name {@code ParityRunner.actionFromName} knows.
     *
     * <p>The give was declared as a plain MOVE for the whole campaign because two vocabularies
     * disagreed -- Rust's enumeration says {@code HandOff}, the harness's action names say
     * {@code HandOver} -- and {@code moveVariant} matched only the first, passing the second
     * through unchanged into a {@code default:} arm. Nothing in the scoring could show it: the
     * agent picked the give correctly and the declaration threw it away.
     *
     * <p>Asserting the mapping is not enough on its own, so this also pins the property that made
     * the bug possible: a name that reaches the declaration must not be one the runner has to
     * guess at.
     */
    @Test
    void moveVariantMapsBothSpellingsOfEveryBallAction() {
        assertEquals("HandOffMove", ActivationChoice.moveVariant("HandOff"));
        assertEquals("HandOffMove", ActivationChoice.moveVariant("HandOver"),
            "the harness spells the give HandOver; it must map to the same declaration");
        assertEquals("PassMove", ActivationChoice.moveVariant("Pass"));

        // Actions with no movement phase are declared as themselves.
        for (String pac : new String[] {"Move", "StandUp", "Block", "Blitz", "StandUpBlitz",
                "Foul", "HailMaryPass"}) {
            assertEquals(pac, ActivationChoice.moveVariant(pac),
                pac + " has no move-variant and must pass through unchanged");
        }
    }

    /**
     * Regression guard for ITER56: Rust's two COVERAGE terms, which this port fed a hardcoded
     * {@code 0.0f} at every one of its six candidate-building call sites.
     *
     * <p>Both are dead below {@code tempScale} 0.1 and live above it, which is exactly why the
     * argmax gate never noticed — and why {@code --heur-scale 1.0} was **0/100**. They are a
     * coverage device that costs play strength by construction, so they are switched off in the
     * sharp arms; a port that drops them is invisible until the sampling arms are gated.
     *
     * <p>Pinned here rather than in a golden because the activation golden was emitted with both
     * at zero and therefore cannot see them.
     */
    @Test
    void coverageTermsAreDeadAtArgmaxAndLiveWhenSampling() {
        java.util.Map<Long, Integer> buckets = new java.util.HashMap<>();
        java.util.Map<String, Integer> actions = new java.util.HashMap<>();

        // Argmax: both terms are zero no matter what the maps hold.
        ActivationChoice.Coverage sharp = new ActivationChoice.Coverage(0.0f, buckets, actions);
        assertEquals(0.0f, sharp.novelty(1234L));
        assertEquals(0.0f, sharp.floor("Move"));

        ActivationChoice.Coverage soft = new ActivationChoice.Coverage(1.0f, buckets, actions);
        // An unseen bucket pays the novelty bonus; a seen one does not.
        assertEquals(0.08f, soft.novelty(1234L));
        soft.record(1234L, "Move");
        assertEquals(0.0f, soft.novelty(1234L), "the bonus is for the FIRST visit only");
        assertEquals(0.08f, soft.novelty(9999L), "a different bucket is still new");

        // The floor decays linearly over four uses and then stays at zero.
        ActivationChoice.Coverage f2 = new ActivationChoice.Coverage(1.0f,
            new java.util.HashMap<>(), new java.util.HashMap<>());
        float[] expected = {0.35f, 0.2625f, 0.175f, 0.0875f, 0.0f, 0.0f};
        for (int seen = 0; seen < expected.length; seen++) {
            assertEquals(expected[seen], f2.floor("Block"), 1e-6f,
                "floor after " + seen + " uses");
            f2.record(0L, "Block");
        }
        // Counters are per ACTION, so one action's use does not lower another's floor.
        assertEquals(0.35f, f2.floor("Foul"));
    }

    /**
     * Regression guard for ITER57: EVERY action branch of {@link ActivationChoice#choose} must
     * consume the coverage floor.
     *
     * <p>ITER56 wired the floor and the novelty bonus into the candidate builders and missed ONE of
     * the six call sites — the FOUL branch, whose arguments wrap across lines differently from the
     * other five, so the edit that replaced {@code 0.0f, 0.0f} did not match it. The result was a
     * foul scored at its raw weight (0.0037) where Rust floored it at 0.35 (0.322): an
     * eighty-fold difference on the one action that was still wrong, and
     * {@code --heur-scale 1.0} sat at 6/100 instead of 94/100.
     *
     * <p>The test drives the real {@code choose} with a stub board offering exactly one foul target
     * and one move, at a raw foul weight far below the floor. With coverage OFF the move wins; with
     * coverage ON the floor lifts the foul above it. A branch that ignores the floor cannot pass
     * both halves.
     */
    @Test
    void everyActionBranchConsumesTheCoverageFloor() {
        // One home player, one prone away player next to him: a legal foul and a legal move.
        List<Features.Snap> snaps = new ArrayList<>();
        snaps.add(new Features.Snap(true, 12, 7, true, true, 6, 3, 1));
        snaps.add(new Features.Snap(false, 13, 7, false, true, 6, 3, 1));
        Features f = Features.build(snaps,
            new Features.BoardState(new FieldCoordinate(2, 2), false, false, false, false), true);

        ActivationChoice.Board board = new ActivationChoice.Board() {
            @Override
            public List<PlanBuilder.BlockTarget> blockTargets(String playerId) {
                return new ArrayList<>();
            }

            @Override
            public List<PlanBuilder.BlockTarget> blitzFoes(String playerId) {
                return new ArrayList<>();
            }

            @Override
            public List<PlanBuilder.BlockTarget> foulTargets(String playerId) {
                List<PlanBuilder.BlockTarget> out = new ArrayList<>();
                // A raw weight far below the 0.35 floor, so the floor decides the outcome.
                out.add(new PlanBuilder.BlockTarget("away_01", new FieldCoordinate(13, 7), 0.004f));
                return out;
            }

            @Override
            public List<PlanBuilder.Receiver> receivers(String playerId, boolean forPass) {
                return new ArrayList<>();
            }
        };

        List<ActivationChoice.Eligible> elig = new ArrayList<>();
        elig.add(new ActivationChoice.Eligible("home_01", 1, new FieldCoordinate(12, 7), true,
            false, java.util.Arrays.asList("Move", "Foul"), 6, 3, 3,
            false, false, false, false, false));

        // Move is already well used, so its floor has decayed to zero; Foul is untouched, so its
        // floor is the full 0.35. That asymmetry is what makes the two branches distinguishable —
        // with a shared floor they would simply tie.
        java.util.Map<String, Integer> seenAction = new java.util.HashMap<>();
        seenAction.put("Move", 4);

        assertEquals("Move", decide(f, board, elig, 0.0f, seenAction),
            "below tempScale 0.1 both coverage terms are dead, so the raw foul weight loses");
        assertEquals("Foul", decide(f, board, elig, 1.0f, seenAction),
            "the FOUL branch must consume the floor like every other branch");
    }

    /** Run {@code choose} at argmax with the coverage terms forced on or off. */
    private static String decide(Features f, ActivationChoice.Board board,
            List<ActivationChoice.Eligible> elig, float coverageScale,
            java.util.Map<String, Integer> seenAction) {
        // A bucket already seen, so `novelty` is zero and the FLOOR is the only term in play.
        java.util.Map<Long, Integer> seenBucket = new java.util.HashMap<>();
        seenBucket.put(12345L, 1);
        ActivationChoice.Coverage cov = new ActivationChoice.Coverage(
            coverageScale, seenBucket, new java.util.HashMap<>(seenAction));
        // Sampler at 0.0 = argmax, so the decision is a fact about the weights, not a draw.
        ActivationChoice.Decision d = ActivationChoice.choose(f, new Sampler(5, 0.0f), board, elig,
            3, false, null, new HashSet<>(), true, false, false, cov, 12345L);
        return d.pac;
    }
}
