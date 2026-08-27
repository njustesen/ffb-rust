package com.fumbbl.ffb.ai.parity.heuristic;

/**
 * The prompt classes the heuristic agent can score, one bit each in a {@link ClassMask}.
 *
 * <p>Mirror of the Rust {@code PromptClass} in {@code agent/heuristic_agent.rs}. <b>The ordinal
 * values are the bit positions and are part of the contract</b> — they must match Rust exactly, or
 * a {@code --heur-classes} spelling would switch on a different class on each side.
 *
 * <p>This exists for the Java-port ladder (see {@code docs/PARITY_HEURISTIC_CAMPAIGN.md}). Porting
 * the whole scorer and only then discovering whether the two engines agree would mean debugging
 * every prompt class at once; the mask lets exactly one class at a time move from "answered by the
 * random parity contract" to "scored by the heuristic".
 */
public enum PromptClass {
    COIN_CHOICE(0, "coin"),
    RECEIVE_CHOICE(1, "receive"),
    KICK_BALL(2, "kick"),
    TOUCHBACK(3, "touchback"),
    TEAM_SETUP(4, "setup"),
    FOLLOW_UP(5, "followup"),
    RE_ROLL_OFFER(6, "reroll"),
    SKILL_USE(7, "skill"),
    INTERCEPTION(8, "intercept"),
    BLOCK_CHOICE(9, "blockchoice"),
    PUSHBACK(10, "pushback"),
    BLOCK_TARGET(11, "blocktarget"),
    BLITZ_TARGET(12, "blitztarget"),
    ACTIVATE_PLAYER(13, "activate"),
    MOVE(14, "move"),
    /** Everything the heuristic does not score itself; always delegated. */
    OTHER(15, "other");

    private final int bit;
    private final String name;

    PromptClass(int bit, String name) {
        this.bit = bit;
        this.name = name;
    }

    /** Bit position in a {@link ClassMask}. Must match the Rust discriminant. */
    public int bit() {
        return bit;
    }

    /** The {@code --heur-classes} spelling. Must match Rust's {@code PromptClass::name}. */
    public String className() {
        return name;
    }

    public static PromptClass byName(String n) {
        for (PromptClass c : values()) {
            if (c.name.equals(n)) {
                return c;
            }
        }
        return null;
    }
}
