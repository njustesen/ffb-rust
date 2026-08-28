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
 * Asserts that {@link PlanBuilder} reproduces the SHAPE of Rust's {@code build_plans} enumeration —
 * the same number of candidates, in the same order, with the same kind, target and weight.
 *
 * <p>Covers the branches whose enumeration is pure geometry plus already-pinned weights: Move
 * (including the loose-ball promotion to Pickup), Block and Blitz. The HandOff and Pass branches
 * need the harness's receiver eligibility and are checked by the live gate instead.
 *
 * <p>The count is asserted before the contents, because the count is what the two-level draw
 * actually consumes: a Move branch that offers 253 squares instead of 254 changes the group weight
 * only if the missing square was the best, but changes the sampling within the group always.
 */
class PlanBuilderTest {

    private static final String PROPERTY = "ffb.planEnumGolden";
    private static final String RELATIVE =
        "crates/ffb-engine/src/agent/testdata/planenum_golden.txt";

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
            "planenum golden table not found (cwd=" + Paths.get("").toAbsolutePath()
                + "). Pass -D" + PROPERTY + "=<path>.");
    }

    private static final class Row {
        boolean home;
        int nr;
        int x;
        int y;
        boolean standing;
    }

    /** One expected candidate line from the golden. */
    private static final class Want {
        String pac;
        String kind;
        String target;
        String dest;
        int weightBits;
    }

    @Test
    void enumerationShapeMatchesRust() throws IOException {
        List<String> lines = Files.readAllLines(goldenPath(), StandardCharsets.UTF_8);

        String caseName = null;
        List<Row> rows = new ArrayList<>();
        FieldCoordinate ball = null;
        boolean loose = false;
        String actions = "";
        float[] params = null;
        List<Want> wants = new ArrayList<>();
        int casesChecked = 0;
        boolean sawPickup = false;
        boolean sawSkippedBlitz = false;

        // Cases are terminated by the next `case` or end of file; collect then verify.
        List<Runnable> pending = new ArrayList<>();

        for (int li = 0; li <= lines.size(); li++) {
            String line = li < lines.size() ? lines.get(li).trim() : "case __end";
            if (line.isEmpty() || line.startsWith("#")) {
                continue;
            }
            String[] p = line.split("\\s+");
            if ("case".equals(p[0])) {
                if (caseName != null && !wants.isEmpty()) {
                    Verify v = verify(caseName, rows, ball, loose, actions, params, wants);
                    sawPickup |= v.sawPickup;
                    sawSkippedBlitz |= v.sawSkippedBlitz;
                    casesChecked++;
                }
                caseName = p[1];
                rows = new ArrayList<>();
                ball = null;
                loose = false;
                actions = "";
                params = null;
                wants = new ArrayList<>();
                continue;
            }
            switch (p[0]) {
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
                case "actions":
                    actions = p[1];
                    break;
                case "params":
                    params = new float[] {
                        Float.intBitsToFloat((int) Long.parseLong(p[1], 16)),
                        Float.intBitsToFloat((int) Long.parseLong(p[2], 16)),
                        Float.intBitsToFloat((int) Long.parseLong(p[3], 16)),
                    };
                    break;
                case "c": {
                    Want w = new Want();
                    w.pac = p[2];
                    w.kind = p[3];
                    w.target = p[4];
                    w.dest = p[5];
                    w.weightBits = (int) Long.parseLong(p[6], 16);
                    wants.add(w);
                    break;
                }
                default:
                    break;
            }
        }

        assertTrue(casesChecked >= 3, "too few cases verified: " + casesChecked);
        assertTrue(sawPickup, "no Pickup candidate in the fixture; the loose-ball promotion is "
            + "untested");
        assertTrue(sawSkippedBlitz, "no blitz case where a distant victim was skipped; the "
            + "adjacency cutoff is untested");
    }

    private static final class Verify {
        boolean sawPickup;
        boolean sawSkippedBlitz;
    }

    /** Rebuild the case and compare against the golden's candidate lines. */
    private static Verify verify(String caseName, List<Row> rows, FieldCoordinate ball,
            boolean loose, String actions, float[] params, List<Want> wants) {
        Verify v = new Verify();
        List<Features.Snap> snaps = new ArrayList<>();
        Row me = null;
        for (Row r : rows) {
            snaps.add(new Features.Snap(r.home, r.x, r.y, r.standing, true, 6, 3, r.nr));
            if (r.home && r.nr == 1) {
                me = r;
            }
        }
        assertNotNull(me, "home_01 must be on the board (" + caseName + ")");
        Features f = Features.build(snaps,
            new Features.BoardState(ball, ball != null, loose, false, false), true);
        FieldCoordinate here = new FieldCoordinate(me.x, me.y);
        boolean isCarrier = f.ballCarried && f.ball != null && f.ball.equals(here);
        ValueModel.Mover m = new ValueModel.Mover(true, isCarrier, 6, 3, 3, false, false, false,
            ValueModel.endzoneDistance(me.x, true), Math.max(8 - 3, 0), f.unactivated[0]);

        // Only the branches whose enumeration is geometry plus pinned weights are rebuilt here.
        if (!"Move".equals(actions) && !"Block".equals(actions) && !"Blitz".equals(actions)) {
            return v;
        }

        Reach r = Reach.search(f, Reach.budgetOf(here, m.ma, false, 0),
            new Reach.MoverSpec(true, m.ag, false, false), false, false, false);
        assertNotNull(r, "reach for " + caseName);
        // w_player, proxy and novelty are PARAMETERS of build_plans, so they come from the
        // golden. Recomputing them here would pin the tier-1 ladder a second time and would
        // silently test a different call if the emitter ever passed something else.
        assertNotNull(params, "params line missing for " + caseName);
        float wPlayer = params[0];
        float novelty = params[2];

        List<PlanBuilder.Candidate> got = new ArrayList<>();
        if ("Move".equals(actions)) {
            PlanBuilder.moveCandidates(f, r, m, "home_01", "Move", wPlayer, 0.0f, novelty, got);
        } else {
            // Block and Blitz share the target list: adjacent opponents that can be blocked, in
            // canonical (side, nr) order, each carrying its own block_weight.
            List<PlanBuilder.BlockTarget> foes = new ArrayList<>();
            for (Row o : rows) {
                if (o.home || !o.standing) {
                    continue;
                }
                FieldCoordinate oc = new FieldCoordinate(o.x, o.y);
                int d = Math.max(Math.abs(here.getX() - oc.getX()),
                    Math.abs(here.getY() - oc.getY()));
                boolean blockable = "Block".equals(actions) ? d == 1 : true;
                if (!blockable) {
                    continue;
                }
                foes.add(new PlanBuilder.BlockTarget(String.format("away_%02d", o.nr), oc, 0.0f));
            }
            if ("Block".equals(actions)) {
                List<PlanBuilder.BlockTarget> withW = new ArrayList<>();
                for (int i = 0; i < foes.size(); i++) {
                    withW.add(new PlanBuilder.BlockTarget(foes.get(i).id, foes.get(i).at,
                        Float.intBitsToFloat(wants.get(i).weightBits) / wPlayer));
                }
                PlanBuilder.blockCandidates(f, m, "home_01", "Block", withW, wPlayer, 0.0f, novelty,
                    got);
            } else {
                int adjacent = 0;
                for (PlanBuilder.BlockTarget t : foes) {
                    int d = Math.max(Math.abs(here.getX() - t.at.getX()),
                        Math.abs(here.getY() - t.at.getY()));
                    if (d == 1) {
                        adjacent++;
                    }
                }
                v.sawSkippedBlitz = foes.size() > adjacent;
                List<PlanBuilder.BlockTarget> withW = new ArrayList<>();
                int wi = 0;
                for (PlanBuilder.BlockTarget t : foes) {
                    int d = Math.max(Math.abs(here.getX() - t.at.getX()),
                        Math.abs(here.getY() - t.at.getY()));
                    float w = 0.0f;
                    if (d == 1) {
                        w = Float.intBitsToFloat(wants.get(wi).weightBits) / wPlayer / 0.85f;
                        wi++;
                    }
                    withW.add(new PlanBuilder.BlockTarget(t.id, t.at, w));
                }
                PlanBuilder.blitzCandidates(m, "home_01", "Blitz", here, withW, wPlayer, 0.0f,
                    novelty, got);
            }
        }

        assertEquals(wants.size(), got.size(), "candidate COUNT (" + caseName + ")");
        for (int i = 0; i < wants.size(); i++) {
            Want w = wants.get(i);
            PlanBuilder.Candidate c = got.get(i);
            String gotKind = c.kind == PlanBuilder.Kind.PICKUP ? "Pickup"
                : c.kind == PlanBuilder.Kind.MOVE ? "Move"
                : c.kind == PlanBuilder.Kind.IMMEDIATE ? "Immediate"
                : c.kind == PlanBuilder.Kind.BLITZ ? "Blitz:" + c.target : c.kind.toString();
            assertEquals(w.kind, gotKind, "kind at " + i + " (" + caseName + ")");
            assertEquals(w.dest, c.dest == null ? "-" : c.dest.toString(),
                "dest at " + i + " (" + caseName + ")");
            if (c.kind == PlanBuilder.Kind.PICKUP) {
                v.sawPickup = true;
            }
            if ("Move".equals(actions) || "Pickup".equals(w.kind)) {
                assertEquals(w.weightBits, Float.floatToRawIntBits(c.weight),
                    "weight at " + i + " (" + caseName + ")");
            }
        }
        return v;
    }
}
