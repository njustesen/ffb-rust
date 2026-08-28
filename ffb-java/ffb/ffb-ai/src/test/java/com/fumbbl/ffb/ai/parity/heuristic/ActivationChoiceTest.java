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
                    ActivationChoice.Decision d = ActivationChoice.choose(f, s, bd, elig, 3, false,
                        null, new HashSet<>(), true, false, false);

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
}
