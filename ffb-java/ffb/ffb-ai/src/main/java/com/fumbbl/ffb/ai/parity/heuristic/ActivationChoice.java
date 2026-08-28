package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;

import com.fumbbl.ffb.model.Game;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

/**
 * Rust {@code handle_activate}, end to end: rank every eligible player, search for the best of
 * them, enumerate their plans, and draw one.
 *
 * <p>Every part of this has its own fixture. What lives here is the COMPOSITION, and a composition
 * bug is invisible to the part fixtures by construction — each of them can be perfectly right while
 * the wiring between two of them is wrong. The order the pieces run in is the contract:
 *
 * <ol>
 *   <li>score every eligible player search-free ({@link Activation#playerWeight});
 *   <li>rank, and give a reach search ONLY to the top {@link Activation#TIER2};
 *   <li>enumerate plans per player IN THE RANKED-CANDIDATE ORDER — actually in the CANONICAL order,
 *       because {@code build_plans} is called walking the canonically-sorted list, not the ranked
 *       one. The ranking decides who gets a SEARCH; it does not reorder the candidate list;
 *   <li>group by declaration and draw two levels.
 * </ol>
 *
 * <p>That third point is the one worth stating twice: Rust sorts canonically, ranks into a separate
 * index vector, and then iterates the CANONICAL list while consulting the rank only to decide
 * whether a search runs. Iterating in ranked order instead would produce the same set of candidates
 * in a different sequence — and the declaration grouping is positional.
 */
public final class ActivationChoice {

    private ActivationChoice() {
    }

    /** One eligible player as the harness reports him. */
    public static final class Eligible {
        public final String id;
        public final int nr;
        public final FieldCoordinate at;
        public final boolean standing;
        public final boolean negatrait;
        /** The actions still live for him this turn, in the engine's order. */
        public final List<String> actions;
        /** His own attributes; the mover model reads all of them. */
        public final int ma;
        public final int ag;
        public final int st;
        public final boolean sureHands;
        public final boolean sideStep;
        public final boolean hasCatch;
        public final boolean dodge;
        public final boolean sureFeet;

        public Eligible(String id, int nr, FieldCoordinate at, boolean standing, boolean negatrait,
                List<String> actions, int ma, int ag, int st, boolean sureHands, boolean sideStep,
                boolean hasCatch, boolean dodge, boolean sureFeet) {
            this.id = id;
            this.nr = nr;
            this.at = at;
            this.standing = standing;
            this.negatrait = negatrait;
            this.actions = actions;
            this.ma = ma;
            this.ag = ag;
            this.st = st;
            this.sureHands = sureHands;
            this.sideStep = sideStep;
            this.hasCatch = hasCatch;
            this.dodge = dodge;
            this.sureFeet = sureFeet;
        }
    }

    /** What the agent decided. {@code player == null} means EndTurn. */
    public static final class Decision {
        public final String player;
        public final String action;
        public final String target;
        /** What the activation is FOR; the follow-up prompts replay this instead of re-deciding. */
        public final PlanBuilder.Kind kind;
        /** Destination cell for a move-shaped plan, else null. */
        public final Integer dest;
        /**
         * The BASE action name, before {@link #moveVariant}. Rust's coverage counter keys on its
         * `PlayerActionChoice` rather than on the declaration, so a give counts as one "HandOver"
         * however it is declared.
         */
        public final String pac;

        public Decision(String player, String action, String target, PlanBuilder.Kind kind,
                Integer dest) {
            this(player, action, target, kind, dest, action);
        }

        public Decision(String player, String action, String target, PlanBuilder.Kind kind,
                Integer dest, String pac) {
            this.pac = pac;
            this.player = player;
            this.action = action;
            this.target = target;
            this.kind = kind;
            this.dest = dest;
        }
    }

    /** Rust {@code move_variant}: a ball action is declared in its MOVE form. */
    public static String moveVariant(String pac) {
        // BOTH spellings of the give. Rust's enumeration calls it `HandOff`; the harness's own
        // action vocabulary, which is what actually reaches this method, calls it `HandOver`
        // (`ParityRunner.nameForAgent(PlayerAction.HAND_OVER)`). Matching only Rust's spelling let
        // `HandOver` fall through unchanged, and `actionFromName` has no case for it -- so its
        // `default` turned every give into a plain MOVE. The agent picked the give (bb2025 seed 2
        // step 49: the SAME candidate at the SAME weight as Rust, index 849 of 2,171) and the
        // harness then declared something else, which is why no scoring diff could show it.
        if ("HandOff".equals(pac) || "HandOver".equals(pac)) {
            return "HandOffMove";
        }
        if ("Pass".equals(pac)) {
            return "PassMove";
        }
        return pac;
    }

    /**
     * Everything the enumeration needs that only the caller can supply — which players may be
     * blocked or fouled, and who may receive a throw. Those are eligibility questions the harness
     * answers from the engine.
     */
    public interface Board {
        List<PlanBuilder.BlockTarget> blockTargets(String playerId);

        List<PlanBuilder.BlockTarget> blitzFoes(String playerId);

        List<PlanBuilder.BlockTarget> foulTargets(String playerId);

        List<PlanBuilder.Receiver> receivers(String playerId, boolean forPass);
    }

    /**
     * @param teamReRoll whether a team re-roll is available; it changes the reach search, not the
     *     scoring, because the first roll of a path can be bought back.
     * @param awaitingRun the player who was just thrown the ball, or null.
     */
    /**
     * @param home which side is acting. Everything downstream reads it: the raster index, the
     *     endzone the movers are attacking, and which players count as opponents. Hardcoding it
     *     to home computes every AWAY decision as though it were attacking the wrong end of the
     *     pitch — which is exactly what the first live gate caught.
     */
    /**
     * Rust's two COVERAGE terms, both dead below {@code tempScale} 0.1 and both hardcoded to
     * {@code 0.0f} in this port until ITER56.
     *
     * <p>{@code novelty} adds a flat 0.08 to every candidate the first time the agent sees a board
     * BUCKET (ball zone, carried, turn/3, weather, half). {@code floor(pac)} raises a candidate's
     * action weight to {@code 0.35 * (1 - min(seen/4, 1))} while that action is under-used. Both
     * exist to widen coverage, both cost play strength by construction, and both are switched off
     * in the sharp arms -- which is why argmax never noticed they were missing and
     * {@code --heur-scale 1.0} was 0/100.
     */
    public static final class Coverage {
        private final float tempScale;
        private final Map<Long, Integer> seenBucket;
        private final Map<String, Integer> seenAction;

        public Coverage(float tempScale, Map<Long, Integer> seenBucket,
                Map<String, Integer> seenAction) {
            this.tempScale = tempScale;
            this.seenBucket = seenBucket;
            this.seenAction = seenAction;
        }

        /** Rust: `if temp_scale >= 0.1 && seen_bucket[bucket] == 0 { 0.08 } else { 0.0 }`. */
        float novelty(long bucket) {
            if (tempScale < 0.1f) {
                return 0.0f;
            }
            return seenBucket.getOrDefault(bucket, 0) == 0 ? 0.08f : 0.0f;
        }

        /** Rust {@code coverage_floor}. */
        float floor(String pac) {
            if (tempScale < 0.1f) {
                return 0.0f;
            }
            int seen = seenAction.getOrDefault(pac, 0);
            return 0.35f * (1.0f - Math.min(seen / 4.0f, 1.0f));
        }

        void record(long bucket, String pac) {
            seenBucket.merge(bucket, 1, Integer::sum);
            seenAction.merge(pac, 1, Integer::sum);
        }
    }

    /**
     * Rust {@code bucket}: the coarse board key the novelty bonus is keyed on.
     *
     * <p>`ballz | carried << 6 | turn << 8 | weather << 12 | half << 16`, with the ball zone
     * defaulting to 31 when there is no ball. The weather index is the ordinal of Rust's `Weather`
     * enum, whose order is Sweltering, VerySunny, Nice, PouringRain, Blizzard, Intro.
     */
    public static long bucket(Features f, Game game) {
        long ballz = 31;
        if (f.ball != null) {
            ballz = (long) (f.ball.getX() / 5) * 4 + (f.ball.getY() / 4);
        }
        long carried = f.carrierAt != null ? 1 : 0;
        int turnNr = Math.max(game.getTurnDataHome().getTurnNr(), game.getTurnDataAway().getTurnNr());
        long turn = turnNr / 3;
        long weather = weatherIndex(game.getFieldModel().getWeather());
        long half = Math.max(0, game.getHalf());
        return ballz | (carried << 6) | (turn << 8) | (weather << 12) | (half << 16);
    }

    /** The ordinal of Rust's `Weather` enum, which is NOT the order of the Java enum. */
    private static long weatherIndex(com.fumbbl.ffb.Weather w) {
        if (w == null) {
            return 2;
        }
        switch (w) {
            case SWELTERING_HEAT: return 0;
            case VERY_SUNNY:      return 1;
            case NICE:            return 2;
            case POURING_RAIN:    return 3;
            case BLIZZARD:        return 4;
            default:              return 5;
        }
    }

    public static Decision choose(Features f, Sampler sampler, Board board, List<Eligible> eligible,
            int turnNr, boolean teamReRoll, String awaitingRun, Set<String> usedThisTurn,
            boolean home, boolean bb2016, boolean blizzard, Coverage coverage,
            long bucket) {
        if (eligible.isEmpty()) {
            return new Decision(null, null, null, null, null);
        }
        boolean anyUnused = false;
        for (Eligible e : eligible) {
            if (!usedThisTurn.contains(e.id)) {
                anyUnused = true;
                break;
            }
        }
        // Every eligible player has already had his activation decided this turn. Re-offering them
        // is how the driver livelocks: an activation that ends without moving leaves the engine's
        // eligible list unchanged, so `usedThisTurn` is the only thing that makes progress.
        if (!anyUnused) {
            return new Decision(null, null, null, null, null);
        }

        // ---- tier 1: search-free proxy for every eligible player ----
        List<Eligible> live = new ArrayList<>();
        List<ValueModel.Mover> movers = new ArrayList<>();
        List<Float> proxies = new ArrayList<>();
        List<Float> weights = new ArrayList<>();
        for (Eligible e : eligible) {
            if (usedThisTurn.contains(e.id) || e.actions.isEmpty()) {
                continue;
            }
            int i = Features.ix(e.at.getX(), e.at.getY());
            boolean isCarrier = f.ballCarried && f.ball != null && f.ball.equals(e.at);
            ValueModel.Mover m = new ValueModel.Mover(home, isCarrier, e.ma, e.ag, e.st,
                e.sureHands, e.sideStep, e.hasCatch,
                ValueModel.endzoneDistance(e.at.getX(), home), Math.max(8 - turnNr, 0),
                f.unactivated[Features.sideIdx(home)]);
            float proxy = Plans.proxyValue(f, e.at, m);
            boolean marked = (f.tz[Features.sideIdx(home)][i] & 0xff) > 0;
            float w = Activation.playerWeight(isCarrier, marked,
                Activation.canFetch(f, e.at, m.ma), !e.standing, proxy, e.negatrait,
                e.id.equals(awaitingRun));
            live.add(e);
            movers.add(m);
            proxies.add(proxy);
            weights.add(w);
        }
        if (live.isEmpty()) {
            return new Decision(null, null, null, null, null);
        }

        // Rank to decide who gets a SEARCH. The candidate list itself stays in canonical order.
        List<Activation.Cand> cands = new ArrayList<>();
        for (int i = 0; i < live.size(); i++) {
            cands.add(new Activation.Cand(live.get(i).id, Features.sideIdx(home),
                live.get(i).nr, weights.get(i), proxies.get(i)));
        }
        List<Activation.Cand> ranked = Activation.rank(cands);
        Set<String> searched = new HashSet<>();
        for (int i = 0; i < Activation.tier2Count(ranked.size()); i++) {
            searched.add(ranked.get(i).id);
        }

        // ---- tier 2: one search per top player, shared by every plan ----
        List<PlanBuilder.Candidate> out = new ArrayList<>();
        for (int i = 0; i < live.size(); i++) {
            Eligible e = live.get(i);
            ValueModel.Mover m = movers.get(i);
            float wPlayer = weights.get(i);
            float proxy = proxies.get(i);
            Reach r = null;
            if (searched.contains(e.id)) {
                r = Reach.search(f, Reach.budgetOf(e.at, m.ma, !e.standing, 0),
                    new Reach.MoverSpec(m.home, m.ag, e.dodge, e.sureFeet), bb2016, blizzard,
                    teamReRoll);
            }
            for (String pac : e.actions) {
                switch (pac) {
                    case "Move":
                    case "StandUp":
                        if (r != null) {
                            PlanBuilder.moveCandidates(f, r, m, e.id, pac, wPlayer, coverage.floor(pac), coverage.novelty(bucket),
                                out);
                        } else {
                            // Tier 1: the proxy stands in, discounted for being optimistic.
                            out.add(new PlanBuilder.Candidate(wPlayer * (proxy * 0.8f), e.id, pac,
                                null, PlanBuilder.Kind.MOVE, null));
                        }
                        break;
                    case "Block":
                        PlanBuilder.blockCandidates(f, m, e.id, pac, board.blockTargets(e.id),
                            wPlayer, coverage.floor(pac), coverage.novelty(bucket), out);
                        break;
                    case "Blitz":
                    case "StandUpBlitz":
                        PlanBuilder.blitzCandidates(m, e.id, pac, e.at, board.blitzFoes(e.id),
                            wPlayer, coverage.floor(pac), coverage.novelty(bucket), out);
                        break;
                    case "Foul":
                        PlanBuilder.foulCandidates(e.id, pac, board.foulTargets(e.id), wPlayer,
                            0.0f, 0.0f, out);
                        break;
                    case "HandOver":
                        if (r != null) {
                            PlanBuilder.handOffCandidates(f, r, m, e.id, pac, e.at,
                                board.receivers(e.id, false), wPlayer, coverage.floor(pac), coverage.novelty(bucket), out);
                        }
                        break;
                    case "Pass":
                    case "HailMaryPass":
                        PlanBuilder.passCandidates(r, m, e.id, pac, e.at,
                            board.receivers(e.id, true),
                            r == null ? java.util.Collections.singletonList(
                                Features.ix(e.at.getX(), e.at.getY()))
                                : Plans.runUpSquares(r, m, e.at),
                            wPlayer, coverage.floor(pac), coverage.novelty(bucket), out);
                        break;
                    default:
                        // Everything else is declared and resolved without a movement phase.
                        out.add(new PlanBuilder.Candidate(
                            wPlayer * Math.max(0.40f, coverage.floor(pac))
                                + coverage.novelty(bucket),
                            e.id, pac, null, PlanBuilder.Kind.IMMEDIATE, null));
                        break;
                }
            }
        }
        if (out.isEmpty()) {
            return new Decision(null, null, null, null, null);
        }

        List<Activation.Option> options = new ArrayList<>();
        for (PlanBuilder.Candidate c : out) {
            options.add(new Activation.Option(c.player, c.pac, c.weight));
        }
        int pick = Activation.chooseCandidate(sampler, options);
        if (pick >= out.size()) {
            return new Decision(null, null, null, null, null); // EndTurn
        }
        PlanBuilder.Candidate c = out.get(pick);
        return new Decision(c.player, moveVariant(c.pac), c.target, c.kind, c.dest, c.pac);
    }
}
