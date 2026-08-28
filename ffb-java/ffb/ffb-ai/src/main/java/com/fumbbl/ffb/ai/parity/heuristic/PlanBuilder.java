package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * Rust {@code build_plans} — the ENUMERATION.
 *
 * <p>Every weight this reads is pinned elsewhere. What lives here is the SHAPE of the list: how
 * many candidates each declared action contributes, in what order, with which {@link Kind} and
 * which target. That shape is the input to the two-level draw, so a list that differs by one entry
 * picks a different action even when every weight agrees.
 *
 * <p>Four things a port loses easily, and each is a deliberate decision rather than an oversight:
 *
 * <ul>
 *   <li><b>Move offers EVERY reachable square</b>, weight-ordered — not a top-K. Pruning to the best
 *       arrival probabilities was measured once and was catastrophic (1.76 touchdowns per game down
 *       to 0.19): {@code pArrive} is an admissible BOUND but not an admissible RANKING, because a
 *       one-square shuffle arrives with p = 1.0 while a six-square scoring run arrives with p ≈ 0.3.
 *       The runs that score were cut before they were ever scored.
 *   <li><b>A square holding a loose ball is {@code PICKUP}</b>, not {@code MOVE} — picking the ball
 *       up changes the value model, so the activation may legitimately continue afterwards.
 *   <li><b>Blitz stops at adjacency.</b> A move-then-blitz does not dispatch in this engine build,
 *       so offering one wastes the team's once-per-turn blitz. The branch SKIPS every non-adjacent
 *       victim rather than scoring it lower — the candidate COUNT is the observable difference.
 *   <li><b>An empty reachable set still emits one candidate</b> at weight 0.02, so a player who
 *       cannot move is not silently absent from the declaration list.
 * </ul>
 */
public final class PlanBuilder {

    /** Discount on a blitz against an already-adjacent victim, versus a plain block. */
    private static final float ADJACENT_BLITZ_DISCOUNT = 0.85f;

    private PlanBuilder() {
    }

    /** Rust {@code PlanKind}, as far as the enumeration distinguishes them. */
    public enum Kind {
        MOVE,
        PICKUP,
        BLITZ,
        FOUL,
        PASS,
        HAND_OFF,
        IMMEDIATE
    }

    /** One enumerated (player, plan) candidate. */
    public static final class Candidate {
        public final float weight;
        public final String player;
        /** The declared action, as the engine names it. */
        public final String pac;
        public final String target;
        public final Kind kind;
        /** Destination cell, when the plan is a move to a square. */
        public final Integer dest;

        public Candidate(float weight, String player, String pac, String target, Kind kind,
                Integer dest) {
            this.weight = weight;
            this.player = player;
            this.pac = pac;
            this.target = target;
            this.kind = kind;
            this.dest = dest;
        }
    }

    /**
     * The Move / StandUp branch: every reachable square, weight-ordered, with the loose-ball square
     * promoted to {@link Kind#PICKUP}.
     *
     * @param wPlayer the tier-1 player weight; every candidate is scaled by it.
     * @param floor the coverage floor for this action, from {@code coverage_floor}.
     */
    public static void moveCandidates(Features f, Reach r, ValueModel.Mover m, String player,
            String pac, float wPlayer, float floor, float novelty, List<Candidate> out) {
        List<Plans.Dest> tops = Plans.topMoves(f, r, m, Integer.MAX_VALUE);
        if (tops.isEmpty()) {
            // Not "nothing to offer": a player who cannot move must still appear in the
            // declaration list, or he vanishes from the draw entirely.
            out.add(new Candidate(wPlayer * Math.max(0.02f, floor) + novelty, player, pac, null,
                Kind.MOVE, null));
            return;
        }
        for (Plans.Dest d : tops) {
            int x = d.i % Features.W;
            int y = d.i / Features.W;
            boolean ontoBall = f.ballLoose && f.ball != null
                && f.ball.getX() == x && f.ball.getY() == y;
            out.add(new Candidate(wPlayer * Math.max(d.w, floor) + novelty, player, pac, null,
                ontoBall ? Kind.PICKUP : Kind.MOVE, d.i));
        }
    }

    /**
     * The Block branch: one candidate per adjacent blockable opponent, in canonical order.
     *
     * @param targets adjacent opponents whose state can be blocked, ALREADY in the engine's order —
     *     which the harness computes, since "who may legally be blocked" is an eligibility question
     *     rather than a scoring one.
     */
    public static void blockCandidates(Features f, ValueModel.Mover m, String player, String pac,
            List<BlockTarget> targets, float wPlayer, float floor, float novelty,
            List<Candidate> out) {
        for (BlockTarget t : targets) {
            float w = t.weight;
            out.add(new Candidate(wPlayer * Math.max(w, floor) + novelty, player, pac, t.id,
                Kind.IMMEDIATE, null));
        }
    }

    /** A block/blitz target with its already-computed {@code block_weight}. */
    public static final class BlockTarget {
        public final String id;
        public final FieldCoordinate at;
        public final float weight;

        public BlockTarget(String id, FieldCoordinate at, float weight) {
            this.id = id;
            this.at = at;
            this.weight = weight;
        }
    }

    /**
     * The Blitz branch.
     *
     * <p>Only victims ALREADY adjacent are offered, discounted against a plain block because a
     * blitz spends the team's once-per-turn blitz that another player might use better. Every
     * non-adjacent victim is skipped outright: a move-then-blitz does not dispatch in this engine
     * build (0.2% of 505 measured attempts), so offering one throws the blitz away.
     */
    public static void blitzCandidates(ValueModel.Mover m, String player, String pac,
            FieldCoordinate here, List<BlockTarget> foes, float wPlayer, float floor, float novelty,
            List<Candidate> out) {
        for (BlockTarget foe : foes) {
            int d = Math.max(Math.abs(here.getX() - foe.at.getX()),
                Math.abs(here.getY() - foe.at.getY()));
            if (d != 1) {
                continue;
            }
            float w = foe.weight * ADJACENT_BLITZ_DISCOUNT;
            out.add(new Candidate(wPlayer * Math.max(w, floor) + novelty, player, pac, foe.id,
                Kind.BLITZ, null));
        }
    }

    /** The Foul branch: one candidate per adjacent foulable victim. */
    public static void foulCandidates(String player, String pac, List<BlockTarget> victims,
            float wPlayer, float floor, float novelty, List<Candidate> out) {
        for (BlockTarget v : victims) {
            out.add(new Candidate(wPlayer * Math.max(v.weight, floor) + novelty, player, pac, v.id,
                Kind.FOUL, null));
        }
    }

    /** Squares next to a receiver a HandOverMove considers giving from. */
    public static final int GIVE_SPOTS = 2;

    /**
     * The HandOff branch.
     *
     * <p>The carrier moves FIRST and gives the ball at the end of the run, so every team-mate he
     * can get NEXT TO is a candidate — not just the ones he is already touching, which is all a
     * "legal hand-off receivers" list reports. For each receiver, the squares ADJACENT to him that
     * the reach search actually reached are scored, ordered, and the best {@link #GIVE_SPOTS} kept.
     *
     * <p>A square that is occupied is skipped UNLESS it is where the carrier already stands — he
     * occupies it himself, and "stand still and give" has to remain available.
     *
     * @param receivers team-mates in the order the engine reports them; the caller supplies the
     *     list because who is a legal receiver is an eligibility question.
     * @param handoffWeightAt scores a give from a square to a receiver, or null when it is not on.
     */
    public static void handOffCandidates(Features f, Reach r, ValueModel.Mover m, String player,
            String pac, FieldCoordinate here, List<Receiver> receivers, float wPlayer, float floor,
            float novelty, List<Candidate> out) {
        for (Receiver rcv : receivers) {
            List<Plans.Dest> spots = new ArrayList<>();
            for (int dy = -1; dy <= 1; dy++) {
                for (int dx = -1; dx <= 1; dx++) {
                    if (dx == 0 && dy == 0) {
                        continue;
                    }
                    int nx = rcv.at.getX() + dx;
                    int ny = rcv.at.getY() + dy;
                    if (!Features.onPitch(nx, ny)) {
                        continue;
                    }
                    int j = Features.ix(nx, ny);
                    boolean isHere = nx == here.getX() && ny == here.getY();
                    if (!r.reached(j) || (f.occupied(j) && !isHere)) {
                        continue;
                    }
                    Float w = rcv.weightFrom(new FieldCoordinate(nx, ny));
                    if (w == null) {
                        continue;
                    }
                    spots.add(new Plans.Dest(Plans.risked(w, r.pArrive(j), m), j));
                }
            }
            spots.sort(Comparator.<Plans.Dest>comparingDouble(d -> -d.w).thenComparingInt(d -> d.i));
            int taken = 0;
            for (Plans.Dest d : spots) {
                if (taken >= GIVE_SPOTS) {
                    break;
                }
                taken++;
                out.add(new Candidate(wPlayer * Math.max(d.w, floor) + novelty, player, pac,
                    rcv.id, Kind.HAND_OFF, d.i));
            }
        }
    }

    /**
     * The Pass branch: the same shape with the throw at the end of the run-up.
     *
     * <p>Enumerated receivers × run-up squares, in that nesting — receivers OUTSIDE, squares
     * inside, which is the order the candidate list ends up in and therefore the order the
     * declaration grouping sees.
     *
     * <p>{@code risked} is folded in only when the throw happens somewhere other than where the
     * thrower stands: standing still carries no chance of never arriving.
     */
    public static void passCandidates(Reach r, ValueModel.Mover m, String player, String pac,
            FieldCoordinate here, List<Receiver> receivers, List<Integer> runUpSquares,
            float wPlayer, float floor, float novelty, List<Candidate> out) {
        for (Receiver rcv : receivers) {
            for (int j : runUpSquares) {
                FieldCoordinate from = new FieldCoordinate(j % Features.W, j / Features.W);
                Float w = rcv.weightFrom(from);
                if (w == null) {
                    continue;
                }
                float weight = w;
                if (r != null && !(from.getX() == here.getX() && from.getY() == here.getY())) {
                    weight = Plans.risked(w, r.pArrive(j), m);
                }
                out.add(new Candidate(wPlayer * Math.max(weight, floor) + novelty, player, pac,
                    rcv.id, Kind.PASS, j));
            }
        }
    }

    /**
     * A receiver, with a callback that prices a give or a throw FROM a given square.
     *
     * <p>The callback exists because the price depends on where the thrower ends up, and the
     * caller is the only one that can compute it — {@code passWeight} needs the engine's pass
     * mechanics and {@code handoffWeight} needs the receiver's own attributes.
     */
    public abstract static class Receiver {
        public final String id;
        public final FieldCoordinate at;

        protected Receiver(String id, FieldCoordinate at) {
            this.id = id;
            this.at = at;
        }

        /** @return the weight, or null when this is not a legal give/throw from there. */
        public abstract Float weightFrom(FieldCoordinate from);
    }

    /** Sort helper mirroring Rust's canonical `(side, nr)` ordering for candidate id lists. */
    public static void sortCanonically(List<BlockTarget> targets,
            java.util.function.ToIntFunction<String> sideOf,
            java.util.function.ToIntFunction<String> nrOf) {
        targets.sort(Comparator
            .comparingInt((BlockTarget t) -> sideOf.applyAsInt(t.id))
            .thenComparingInt(t -> nrOf.applyAsInt(t.id)));
    }

    /** Convenience for the enumeration order used by tests. */
    public static List<Candidate> empty() {
        return new ArrayList<>();
    }
}
