package com.fumbbl.ffb.server.mechanic;

/**
 * Documented modifier values for skills that affect armor rolls.
 *
 * Armor roll formula: total = d6 + d6 + sum(applicable modifiers)
 * Armor broken when:
 *   BB2016: total > armour  (strict)
 *   BB2020/BB2025: total >= armour  (inclusive)
 *
 * Skills that affect armor and injury rolls are mutually exclusive (apply to one, not both).
 */
public final class ArmorModifierValues {

    /**
     * Mighty Blow default modifier for armor rolls.
     * BB2016: static +1. BB2020/BB2025: configurable, default +1.
     */
    public static final int MIGHTY_BLOW_DEFAULT = 1;

    /**
     * Dirty Player default modifier for armor rolls (foul actions only).
     * BB2016: static +1. BB2020/BB2025: configurable, default +1.
     */
    public static final int DIRTY_PLAYER_DEFAULT = 1;

    /**
     * Piling On modifier for armor rolls.
     * Constant +2 in all editions.
     */
    public static final int PILING_ON = 2;

    /**
     * Stunty imposes -1 to the armor roll against a Stunty player (attacker's roll).
     * Used in armor roll context when defender has the Stunty property.
     */
    public static final int STUNTY = -1;

    /**
     * Fixed armour cap applied by Chainsaw and similar skills (BB2016).
     * Any armour above this value is treated as this value.
     */
    public static final int FIXED_ARMOUR_CAP_BB2016 = 7;

    /**
     * Fixed armour cap applied by Chainsaw and similar skills (BB2020/BB2025).
     */
    public static final int FIXED_ARMOUR_CAP_BB2020 = 8;

    private ArmorModifierValues() {}
}
