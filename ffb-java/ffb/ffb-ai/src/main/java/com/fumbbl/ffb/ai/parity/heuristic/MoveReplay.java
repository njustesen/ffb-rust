package com.fumbbl.ffb.ai.parity.heuristic;

/**
 * Rust {@code replay_plan} — the plan-replay state machine.
 *
 * <p>Seven exits and four engine guards. It is a pure function of the board facts the caller
 * gathers, which is what makes it testable at all: the Rust original was control flow tangled with
 * the mutations it drives, and there was no way to call it with made-up inputs.
 *
 * <p>The rules, in the order they are applied — and the order IS the content:
 *
 * <ol>
 *   <li>An EMPTY square list means no MOVEMENT is left, <b>not</b> that there is nothing to do. A
 *       pending give, throw, blitz or foul still has to be sent. Bailing here threw away every give
 *       whose run-up spent the carrier's whole move, which is most of the good ones.
 *   <li>A path is delivered only when the offered squares contain its next step. If the board moved
 *       under the plan, re-decide rather than insist.
 *   <li>Every terminal action is gated on the ENGINE's own condition and latched with
 *       {@code fired}, because {@code StepInitMoving} re-emits this prompt when its guard fails —
 *       so resending a rejected action would spin forever.
 *   <li>A delivered plain move ends: moving twice reaches the same square. A plan that never
 *       carried a path (a tier-1 proxy pick) still has to decide, so it falls through instead.
 *   <li>Once fired, only a BLITZ has anything legitimate left — its post-block movement — and a
 *       PICKUP re-decides, because picking the ball up genuinely changed the value model.
 * </ol>
 */
public final class MoveReplay {

    private MoveReplay() {
    }

    /** What the replay decided, before it is turned into a command. */
    public enum Verdict {
        /** Deliver the whole remaining path in one answer; the engine walks it. */
        DELIVER_PATH,
        /** Send the plan's terminal action. */
        FIRE_TERMINAL,
        END_PLAYER_ACTION,
        /** No usable plan — re-decide from scratch. */
        REPLAN
    }

    /** The plan kinds, as far as the replay distinguishes them. */
    public enum Kind {
        MOVE,
        PICKUP,
        IMMEDIATE,
        BLITZ,
        FOUL,
        PASS,
        HAND_OFF
    }

    /** The board facts the decision reads, gathered once by the caller. */
    public static final class Facts {
        /**
         * The action the ENGINE currently has on the acting player, which gates every terminal
         * dispatch. Null when there is none.
         */
        public final String paNow;
        public final boolean hasBlocked;
        public final boolean hasFouled;
        public final boolean targetAdjacent;
        public final boolean targetOnPitch;
        public final boolean squaresIncludeNext;
        public final boolean squaresEmpty;

        public Facts(String paNow, boolean hasBlocked, boolean hasFouled, boolean targetAdjacent,
                boolean targetOnPitch, boolean squaresIncludeNext, boolean squaresEmpty) {
            this.paNow = paNow;
            this.hasBlocked = hasBlocked;
            this.hasFouled = hasFouled;
            this.targetAdjacent = targetAdjacent;
            this.targetOnPitch = targetOnPitch;
            this.squaresIncludeNext = squaresIncludeNext;
            this.squaresEmpty = squaresEmpty;
        }
    }

    private static boolean isTerminal(Kind k) {
        return k == Kind.HAND_OFF || k == Kind.PASS || k == Kind.BLITZ || k == Kind.FOUL;
    }

    /** Rust {@code replay_plan}. */
    public static Verdict decide(Kind kind, boolean planIsForThisPlayer, boolean pathEmpty,
            boolean delivered, boolean fired, Facts f) {
        boolean terminalPending = isTerminal(kind);
        // NOTE the asymmetry, and it is Rust's: `terminalPending` is read off the plan WITHOUT
        // checking whose plan it is, so a pending give belonging to another player still suppresses
        // this exit. Tightening it to "this player's plan" is the obvious cleanup and would change
        // behaviour.
        if (f.squaresEmpty && !terminalPending) {
            return Verdict.END_PLAYER_ACTION;
        }
        if (!planIsForThisPlayer) {
            return Verdict.REPLAN;
        }
        if (!pathEmpty) {
            return f.squaresIncludeNext ? Verdict.DELIVER_PATH : Verdict.REPLAN;
        }
        if (!fired) {
            boolean dispatchable;
            switch (kind) {
                case BLITZ:
                    dispatchable = ("BlitzMove".equals(f.paNow) || "KickEmBlitz".equals(f.paNow))
                        && !f.hasBlocked && f.targetAdjacent;
                    break;
                case FOUL:
                    dispatchable = "FoulMove".equals(f.paNow) && !f.hasFouled && f.targetAdjacent;
                    break;
                case PASS:
                    dispatchable = ("PassMove".equals(f.paNow) || "Pass".equals(f.paNow)
                        || "HailMaryPass".equals(f.paNow)) && f.targetOnPitch;
                    break;
                case HAND_OFF:
                    dispatchable = ("HandOverMove".equals(f.paNow) || "HandOver".equals(f.paNow))
                        && f.targetOnPitch;
                    break;
                default:
                    dispatchable = false;
                    break;
            }
            if (dispatchable) {
                return Verdict.FIRE_TERMINAL;
            }
            if ((kind == Kind.MOVE || kind == Kind.IMMEDIATE) && delivered) {
                return Verdict.END_PLAYER_ACTION;
            }
            if (kind != Kind.PICKUP && kind != Kind.MOVE && delivered) {
                return Verdict.END_PLAYER_ACTION;
            }
            return Verdict.REPLAN;
        }
        // Already fired: the activation is over, except a blitz's post-block movement and a pickup.
        if (kind != Kind.PICKUP && kind != Kind.BLITZ) {
            return Verdict.END_PLAYER_ACTION;
        }
        return Verdict.REPLAN;
    }
}
