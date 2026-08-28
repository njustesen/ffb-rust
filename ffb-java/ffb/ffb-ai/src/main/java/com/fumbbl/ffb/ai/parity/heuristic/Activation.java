package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * The tier-1 activation ranking from Rust's {@code handle_activate}.
 *
 * <p>Before any pathfinding runs, every eligible player is scored search-free, and only the top
 * {@link #TIER2} get a Dijkstra at all. So this decides not merely who is likeliest to be picked
 * but <b>who is even considered properly</b> — and a disagreement here does not look like a wrong
 * move. It looks like the right move made by the wrong player, or by a player the other engine
 * never scored.
 *
 * <p>{@link #playerWeight} is a chain of else-ifs, so the ORDER of the rungs can matter as much as
 * the constants — but only where two rungs can hold at once, and most of these cannot. "Can fetch a
 * loose ball" and "is the carrier" are mutually EXCLUSIVE by construction (a loose ball is nobody's
 * possession), so their relative order is unobservable and no board can pin it. The one adjacent
 * pair that CAN both hold is {@code prone && marked} against {@code proxy > 0.25}, and the
 * {@code prone_marked_with_support} board exists to pin exactly that.
 *
 * <p>The negatrait multiplier applies AFTER the ladder, and the {@code awaitingRun} override after
 * that — it OVERWRITES rather than scales, so a player who was just thrown the ball outranks
 * everyone however bad his own rung was.
 */
public final class Activation {

    /**
     * How many players get a real search. Rust's comment is worth carrying: a player without a
     * search contributes ONE placeholder option standing in for his whole destination space, so he
     * collects probability mass that should have been spread over a dozen squares and is
     * over-sampled against players whose destinations are enumerated. 16 covers every player who
     * can ever be eligible.
     */
    public static final int TIER2 = 16;

    private Activation() {
    }

    /** One eligible player, scored before any search. */
    public static final class Cand {
        public final String id;
        /** Canonical ordering key: home before away, then jersey number. Never the id. */
        public final int side;
        public final int nr;
        public final float wPlayer;
        public final float proxy;

        public Cand(String id, int side, int nr, float wPlayer, float proxy) {
            this.id = id;
            this.side = side;
            this.nr = nr;
            this.wPlayer = wPlayer;
            this.proxy = proxy;
        }
    }

    /**
     * Rust's {@code w_player} ladder.
     *
     * @param awaitingRun this player was just thrown the ball; running it on is the reason the
     *     throw was made, so he is forced to 1.0.
     */
    public static float playerWeight(boolean isCarrier, boolean marked, boolean canFetch,
            boolean prone, float proxy, boolean negatrait, boolean awaitingRun) {
        float w;
        if (isCarrier && marked) {
            w = 0.95f;
        } else if (canFetch) {
            w = 0.92f;
        } else if (isCarrier) {
            w = 0.88f;
        } else if (prone && marked) {
            w = 0.70f;
        } else if (proxy > 0.25f) {
            w = 0.45f;
        } else {
            w = 0.30f;
        }
        if (negatrait) {
            w *= 0.55f;
        }
        if (awaitingRun) {
            w = 1.0f;
        }
        return w;
    }

    /**
     * Can this player reach a LOOSE ball at all this activation? Rust measures in Chebyshev steps
     * against {@code MA + 2}, i.e. including both rushes, and only while the ball is loose — a
     * carried ball is somebody's possession, not something to fetch.
     */
    public static boolean canFetch(Features f, FieldCoordinate at, int ma) {
        if (!f.ballLoose || f.ball == null) {
            return false;
        }
        int d = Math.max(Math.abs(at.getX() - f.ball.getX()), Math.abs(at.getY() - f.ball.getY()));
        return d <= ma + 2;
    }

    /**
     * Rank the candidates: {@code wPlayer * max(proxy, 0.05)} descending, canonical key ascending
     * on ties.
     *
     * <p>The floor on {@code proxy} is what stops a player whose proxy is exactly 0 from being
     * unrankable — without it every such player collapses to the same 0 and the tie-break decides
     * everything.
     *
     * <p>Rust sorts canonically FIRST and then ranks with a stable sort, which is the same result
     * as an explicit two-level comparator; this does the latter so it does not depend on the input
     * arriving in canonical order.
     */
    public static List<Cand> rank(List<Cand> cands) {
        List<Cand> out = new ArrayList<>(cands);
        out.sort(Comparator
            .<Cand>comparingDouble(c -> -(c.wPlayer * Math.max(c.proxy, 0.05f)))
            .thenComparingInt(c -> c.side)
            .thenComparingInt(c -> c.nr));
        return out;
    }

    /** One enumerated option: which declaration it belongs to, and what it is worth. */
    public static final class Option {
        public final String player;
        /** The declared action. Only equality matters, so any stable key will do. */
        public final String pac;
        public final float weight;

        public Option(String player, String pac, float weight) {
            this.player = player;
            this.pac = pac;
            this.weight = weight;
        }
    }

    /**
     * Rust {@code group_declarations}: group the options by DECLARATION — the {@code (player,
     * action)} pair the engine actually receives.
     *
     * <p><b>Contiguous runs, not a keyed lookup.</b> {@code build_plans} emits a player's options
     * one action at a time, so a declaration's options are adjacent; grouping by key instead would
     * merge two non-adjacent runs of the same declaration into one group and change the sampling
     * tree. (It is also what made the original O(groups) of string comparison per option.)
     */
    public static List<List<Integer>> groupDeclarations(List<Option> options) {
        List<List<Integer>> groups = new ArrayList<>();
        for (int i = 0; i < options.size(); i++) {
            Option c = options.get(i);
            Option prev = i > 0 ? options.get(i - 1) : null;
            boolean same = prev != null && prev.pac.equals(c.pac) && prev.player.equals(c.player);
            if (same) {
                groups.get(groups.size() - 1).add(i);
            } else {
                List<Integer> g = new ArrayList<>();
                g.add(i);
                groups.add(g);
            }
        }
        return groups;
    }

    /**
     * The two-level draw: pick a DECLARATION at {@code T = 0.18}, then an option within it at
     * {@code T = 0.10}.
     *
     * <p>The agent does not sample flatly, and the reason is cardinality: a Move declaration can
     * carry two thousand destinations and a Block nine, so a flat draw lets the Move branch drown
     * the Block one purely by how many squares exist. Scoring each group by its BEST child keeps
     * argmax identical to a flat draw while fixing the sampled case.
     *
     * <p>{@code EndTurn} is appended as its own group with weight exactly 0.0 — it therefore beats
     * every negative-weight branch and loses to every positive one, which is what "banking what the
     * team has" should mean.
     *
     * <p><b>Two draws, not one</b>, unless a level has a single entry (or the temperature is 0,
     * where nothing is drawn). A singleton group silently costs a draw fewer, and the stream
     * desynchronises from there — which is why the fixture pins the draw COUNT and not only the
     * choice.
     *
     * @return the chosen index into {@code options}, or {@code options.size()} for EndTurn.
     */
    public static int chooseCandidate(Sampler sampler, List<Option> options) {
        List<List<Integer>> groups = groupDeclarations(options);
        int endIdx = options.size();
        List<Integer> endGroup = new ArrayList<>();
        endGroup.add(endIdx);
        groups.add(endGroup);

        float[] allW = new float[options.size() + 1];
        for (int i = 0; i < options.size(); i++) {
            allW[i] = options.get(i).weight;
        }
        allW[endIdx] = 0.0f;

        float[] gw = new float[groups.size()];
        for (int g = 0; g < groups.size(); g++) {
            // Rust folds with f32::MIN as the seed, not negative infinity; identical here because
            // every weight is finite, and written the same way so it stays identical.
            float best = -Float.MAX_VALUE;
            for (int j : groups.get(g)) {
                best = Math.max(best, allW[j]);
            }
            gw[g] = best;
        }
        int gi = sampler.softmaxPick(gw, gw.length, 0.18f);
        List<Integer> chosen = groups.get(gi);
        float[] cw = new float[chosen.size()];
        for (int k = 0; k < chosen.size(); k++) {
            cw[k] = allW[chosen.get(k)];
        }
        int ci = sampler.softmaxPick(cw, cw.length, 0.10f);
        return chosen.get(ci);
    }

    /** The indices into {@link #rank}'s output that get a real search. */
    public static int tier2Count(int nCands) {
        return Math.min(TIER2, nCands);
    }
}
