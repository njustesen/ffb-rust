package com.fumbbl.ffb.server.mechanic;

/**
 * Documented modifier values for skills that affect injury rolls.
 *
 * These constants capture the "what value" side of skill modifiers, independent
 * of the "does it apply here" context logic.
 *
 * Injury roll formula: total = d6 + d6 + sum(applicable modifiers)
 * Outcomes determined by InjuryCalc.interpretInjuryTotalBB2016/BB2020.
 */
public final class InjuryModifierValues {

    /** Mighty Blow default modifier for injury rolls (+1 to injury total). */
    public static final int MIGHTY_BLOW_DEFAULT = 1;

    /** Mighty Blow applies to EITHER armor OR injury roll, never both. */
    public static final boolean MIGHTY_BLOW_EXCLUSIVE = true;

    /** Dirty Player default modifier for injury rolls (+1 to injury total). */
    public static final int DIRTY_PLAYER_DEFAULT = 1;

    /** Dirty Player only applies during foul actions. */
    public static final boolean DIRTY_PLAYER_FOUL_ONLY = true;

    /** Each niggling injury adds this value to the opponent's injury roll (BB2016). */
    public static final int NIGGLING_INJURY_PER_STACK = 1;

    /** Fireball special effect adds this to injury rolls. */
    public static final int FIREBALL_MODIFIER = 1;

    /** Lightning special effect adds this to injury rolls. */
    public static final int LIGHTNING_MODIFIER = 1;

    private InjuryModifierValues() {}
}
