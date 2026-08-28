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
