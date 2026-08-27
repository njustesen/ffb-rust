package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.PriorityQueue;

/**
 * The quantised-key Dijkstra from the Rust {@code reach_with}.
 *
 * <p>Every square gets a KEY, an integer approximation of {@code -ln(p_arrive) * 4096}, accumulated
 * along the cheapest route. The value model turns it back into a probability with
 * {@link #pArrive(int)} and compares those with {@code >}, so this has to agree with Rust exactly
 * and not approximately.
 *
 * <p>Three things make that harder than it looks, and all three are pinned by
 * {@code reach_golden.txt}:
 *
 * <ul>
 *   <li><b>The key increment.</b> {@code (-ln(p) * 4096)} is a float and the key is an int. Rust
 *       clamps before converting because {@code as u32} saturates there and Java's cast does not,
 *       so this does the identical clamp rather than relying on either language's rule.
 *   <li><b>Heap ties.</b> The queue is ordered on {@code (key, cost, idx)}. Ordering on {@code key}
 *       alone would pop equal-key cells in whatever order they were inserted and settle a different
 *       {@code prev} — the same arrival probability by a different ROUTE, which is a real
 *       divergence the moment the agent walks that route.
 *   <li><b>The visit order.</b> {@code order} is sorted before it leaves this class, precisely so
 *       that nothing downstream can depend on how the heap broke a tie.
 * </ul>
 */
public final class Reach {

    /** Rust {@code KEY_SCALE}. */
    public static final float KEY_SCALE = 4096.0f;
    /** Rust {@code NO_PREV}. */
    public static final int NO_PREV = 0xffff;
    /** Rust {@code UNREACHED.key}. */
    public static final long UNREACHED = 0xffffffffL;
    /** Rust {@code STAND_UP_COST} — {@code mechanics/movement.rs}. */
    public static final int STAND_UP_COST = 3;

    /** Where a search starts and how much movement it has left. */
    public static final class Budget {
        public final FieldCoordinate start;
        public final int ma;
        public final int spent;
        public final int cap;
        public final float gate;

        public Budget(FieldCoordinate start, int ma, int spent, int cap, float gate) {
            this.start = start;
            this.ma = ma;
            this.spent = spent;
            this.cap = cap;
            this.gate = gate;
        }
    }

    /** The mover's own attributes, which the search reads but the rasters do not carry. */
    public static final class MoverSpec {
        public final boolean home;
        public final int ag;
        public final boolean dodge;
        public final boolean sureFeet;

        public MoverSpec(boolean home, int ag, boolean dodge, boolean sureFeet) {
            this.home = home;
            this.ag = ag;
            this.dodge = dodge;
            this.sureFeet = sureFeet;
        }
    }

    /** Keys are unsigned 32-bit in Rust; a long holds them without sign trouble. */
    public final long[] key = new long[Features.CELLS];
    public final int[] cost = new int[Features.CELLS];
    public final int[] gfi = new int[Features.CELLS];
    public final int[] prev = new int[Features.CELLS];
    public final boolean[] seen = new boolean[Features.CELLS];
    /** Visited cells other than the start, ASCENDING. */
    public int[] order = new int[0];
    public int start;
    public float gate;

    private Reach() {
        Arrays.fill(key, UNREACHED);
        Arrays.fill(prev, NO_PREV);
    }

    public boolean reached(int i) {
        return key[i] != UNREACHED;
    }

    /** Rust {@code Reach::p_arrive}. */
    public float pArrive(int i) {
        return DetMath.expF32(-((float) key[i]) / KEY_SCALE) * gate;
    }

    /**
     * Rust {@code Reach::path_to} — the back-pointer walk, done once for the destination actually
     * chosen rather than by cloning a path at every improvement.
     */
    public List<FieldCoordinate> pathTo(int i) {
        List<FieldCoordinate> out = new ArrayList<>();
        while (i != start) {
            out.add(new FieldCoordinate(i % Features.W, i / Features.W));
            int p = prev[i];
            if (p == NO_PREV) {
                out.clear();
                return out;
            }
            i = p;
        }
        java.util.Collections.reverse(out);
        return out;
    }

    /** Rust {@code p_roll}: the chance of making a d6 target, clamped to [1/6, 5/6]. */
    public static float pRoll(int target) {
        float p = (7 - target) / 6.0f;
        return Math.min(Math.max(p, 1.0f / 6.0f), 5.0f / 6.0f);
    }

    /** Rust {@code p_with_reroll}. */
    public static float pWithReRoll(float p, float pRr) {
        return p + (1.0f - p) * pRr * p;
    }

    /** Rust {@code dodge_target}. BB2016 uses the old {@code 7 - AG} scale. */
    public static int dodgeTarget(boolean bb2016, int ag, int tzOnDest) {
        if (bb2016) {
            return Math.max((7 - Math.min(ag, 6)) - 1 + tzOnDest, 2);
        }
        return Math.max(ag + tzOnDest, 2);
    }

    /** Rust {@code gfi_target}: base 2, and Blizzard is 3 in EVERY edition. */
    public static int gfiTarget(boolean blizzard) {
        return blizzard ? 3 : 2;
    }

    /** Rust {@code budget_of}. A prone mover pays the stand-up cost, or rolls for it and stays. */
    public static Budget budgetOf(FieldCoordinate start, int maBase, boolean prone, int spent) {
        int ma;
        float gate;
        if (prone) {
            if (maBase <= STAND_UP_COST) {
                ma = 0;
                gate = pRoll(4);
            } else {
                ma = maBase - STAND_UP_COST;
                gate = 1.0f;
            }
        } else {
            ma = maBase;
            gate = 1.0f;
        }
        int cap = Math.max(ma + 2 - spent, 0);
        return new Budget(start, ma, spent, cap, gate);
    }

    private static final class Item {
        final long key;
        final int cost;
        final int idx;

        Item(long key, int cost, int idx) {
            this.key = key;
            this.cost = cost;
            this.idx = idx;
        }
    }

    /**
     * Rust {@code reach_with}.
     *
     * @return null when the mover cannot move at all, mirroring Rust's {@code None}.
     */
    public static Reach search(Features f, Budget b, MoverSpec mover, boolean bb2016,
            boolean blizzard, boolean teamReRoll) {
        if (b.cap <= 0 || !Features.onPitch(b.start.getX(), b.start.getY())) {
            return null;
        }
        Reach r = new Reach();
        r.gate = b.gate;

        int gt = gfiTarget(blizzard);
        int s = Features.sideIdx(mover.home);

        int si = Features.ix(b.start.getX(), b.start.getY());
        r.start = si;
        r.key[si] = 0;
        r.cost[si] = 0;
        r.gfi[si] = 0;
        r.prev[si] = NO_PREV;

        // (key, cost, idx) — idx last, and present ONLY so ties break the same way Rust's do.
        PriorityQueue<Item> heap = new PriorityQueue<>((x, y) -> {
            if (x.key != y.key) {
                return Long.compare(x.key, y.key);
            }
            if (x.cost != y.cost) {
                return Integer.compare(x.cost, y.cost);
            }
            return Integer.compare(x.idx, y.idx);
        });
        heap.add(new Item(0, 0, si));

        List<Integer> order = new ArrayList<>();
        while (!heap.isEmpty()) {
            Item it = heap.poll();
            int i = it.idx;
            if (r.seen[i] || it.key > r.key[i]) {
                continue;
            }
            r.seen[i] = true;
            if (i != si) {
                order.add(i);
            }
            if (r.cost[i] >= b.cap) {
                continue;
            }
            int cx = i % Features.W;
            int cy = i / Features.W;
            boolean leavingTz = (f.tz[s][i] & 0xff) > 0;
            // The team re-roll is worth its full value on the FIRST roll of a path and nothing after.
            boolean firstRoll = r.key[i] == 0;

            for (int dy = -1; dy <= 1; dy++) {
                for (int dx = -1; dx <= 1; dx++) {
                    if (dx == 0 && dy == 0) {
                        continue;
                    }
                    int nx = cx + dx;
                    int ny = cy + dy;
                    if (!Features.onPitch(nx, ny)) {
                        continue;
                    }
                    int j = Features.ix(nx, ny);
                    if (f.occupied(j)) {
                        continue;
                    }
                    int ncost = it.cost + 1;
                    if (ncost > b.cap) {
                        continue;
                    }
                    float pStep = 1.0f;
                    boolean usedRr = false;
                    if (leavingTz) {
                        int t = dodgeTarget(bb2016, mover.ag, f.tz[s][j] & 0xff);
                        float raw = pRoll(t);
                        if (mover.dodge) {
                            pStep *= pWithReRoll(raw, 1.0f);
                        } else if (teamReRoll && firstRoll) {
                            pStep *= pWithReRoll(raw, 1.0f);
                            usedRr = true;
                        } else {
                            pStep *= raw;
                        }
                    }
                    boolean gfiHere = ncost + b.spent > b.ma;
                    if (gfiHere) {
                        float raw = pRoll(gt);
                        if (mover.sureFeet || (teamReRoll && firstRoll && !usedRr)) {
                            pStep *= pWithReRoll(raw, 1.0f);
                        } else {
                            pStep *= raw;
                        }
                    }
                    // The clamp is Rust's, and it is deliberate: `as u32` saturates there and the
                    // Java cast does not, so neither side relies on its own conversion rule.
                    float inc = -DetMath.lnF32(Math.max(pStep, 1e-6f)) * KEY_SCALE;
                    inc = Math.min(Math.max(inc, 0.0f), 1.0e9f);
                    long nkey = it.key + (long) inc;
                    if (nkey < r.key[j]) {
                        r.key[j] = nkey;
                        r.cost[j] = ncost;
                        r.gfi[j] = r.gfi[i] + (gfiHere ? 1 : 0);
                        r.prev[j] = i;
                        r.seen[j] = false;
                        heap.add(new Item(nkey, ncost, j));
                    }
                }
            }
        }

        // The output order must not depend on heap tie-breaking.
        int[] arr = new int[order.size()];
        for (int i = 0; i < arr.length; i++) {
            arr[i] = order.get(i);
        }
        Arrays.sort(arr);
        r.order = arr;
        return r;
    }
}
