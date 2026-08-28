package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
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

        public Eligible(String id, int nr, FieldCoordinate at, boolean standing, boolean negatrait,
                List<String> actions) {
            this.id = id;
            this.nr = nr;
            this.at = at;
            this.standing = standing;
            this.negatrait = negatrait;
            this.actions = actions;
        }
    }

    /** What the agent decided. {@code player == null} means EndTurn. */
    public static final class Decision {
        public final String player;
        public final String action;
        public final String target;

        public Decision(String player, String action, String target) {
            this.player = player;
            this.action = action;
            this.target = target;
        }
    }

    /** Rust {@code move_variant}: a ball action is declared in its MOVE form. */
    public static String moveVariant(String pac) {
        if ("HandOff".equals(pac)) {
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
    public static Decision choose(Features f, Sampler sampler, Board board, List<Eligible> eligible,
            int turnNr, boolean teamReRoll, String awaitingRun, Set<String> usedThisTurn) {
        if (eligible.isEmpty()) {
            return new Decision(null, null, null);
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
            return new Decision(null, null, null);
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
            ValueModel.Mover m = new ValueModel.Mover(true, isCarrier, 6, 3, 3, false, false, false,
                ValueModel.endzoneDistance(e.at.getX(), true), Math.max(8 - turnNr, 0),
                f.unactivated[0]);
            float proxy = Plans.proxyValue(f, e.at, m);
            boolean marked = (f.tz[0][i] & 0xff) > 0;
            float w = Activation.playerWeight(isCarrier, marked,
                Activation.canFetch(f, e.at, m.ma), !e.standing, proxy, e.negatrait,
                e.id.equals(awaitingRun));
            live.add(e);
            movers.add(m);
            proxies.add(proxy);
            weights.add(w);
        }
        if (live.isEmpty()) {
            return new Decision(null, null, null);
        }

        // Rank to decide who gets a SEARCH. The candidate list itself stays in canonical order.
        List<Activation.Cand> cands = new ArrayList<>();
        for (int i = 0; i < live.size(); i++) {
            cands.add(new Activation.Cand(live.get(i).id, 0, live.get(i).nr, weights.get(i),
                proxies.get(i)));
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
                    new Reach.MoverSpec(true, m.ag, false, false), false, false, teamReRoll);
            }
            for (String pac : e.actions) {
                switch (pac) {
                    case "Move":
                    case "StandUp":
                        if (r != null) {
                            PlanBuilder.moveCandidates(f, r, m, e.id, pac, wPlayer, 0.0f, 0.0f,
                                out);
                        } else {
                            // Tier 1: the proxy stands in, discounted for being optimistic.
                            out.add(new PlanBuilder.Candidate(wPlayer * (proxy * 0.8f), e.id, pac,
                                null, PlanBuilder.Kind.MOVE, null));
                        }
                        break;
                    case "Block":
                        PlanBuilder.blockCandidates(f, m, e.id, pac, board.blockTargets(e.id),
                            wPlayer, 0.0f, 0.0f, out);
                        break;
                    case "Blitz":
                    case "StandUpBlitz":
                        PlanBuilder.blitzCandidates(m, e.id, pac, e.at, board.blitzFoes(e.id),
                            wPlayer, 0.0f, 0.0f, out);
                        break;
                    case "Foul":
                        PlanBuilder.foulCandidates(e.id, pac, board.foulTargets(e.id), wPlayer,
                            0.0f, 0.0f, out);
                        break;
                    case "HandOver":
                        if (r != null) {
                            PlanBuilder.handOffCandidates(f, r, m, e.id, pac, e.at,
                                board.receivers(e.id, false), wPlayer, 0.0f, 0.0f, out);
                        }
                        break;
                    case "Pass":
                    case "HailMaryPass":
                        PlanBuilder.passCandidates(r, m, e.id, pac, e.at,
                            board.receivers(e.id, true),
                            r == null ? java.util.Collections.singletonList(
                                Features.ix(e.at.getX(), e.at.getY()))
                                : Plans.runUpSquares(r, m, e.at),
                            wPlayer, 0.0f, 0.0f, out);
                        break;
                    default:
                        // Everything else is declared and resolved without a movement phase.
                        out.add(new PlanBuilder.Candidate(wPlayer * 0.40f, e.id, pac, null,
                            PlanBuilder.Kind.IMMEDIATE, null));
                        break;
                }
            }
        }
        if (out.isEmpty()) {
            return new Decision(null, null, null);
        }

        List<Activation.Option> options = new ArrayList<>();
        for (PlanBuilder.Candidate c : out) {
            options.add(new Activation.Option(c.player, c.pac, c.weight));
        }
        int pick = Activation.chooseCandidate(sampler, options);
        if (pick >= out.size()) {
            return new Decision(null, null, null); // EndTurn
        }
        PlanBuilder.Candidate c = out.get(pick);
        return new Decision(c.player, moveVariant(c.pac), c.target);
    }
}
