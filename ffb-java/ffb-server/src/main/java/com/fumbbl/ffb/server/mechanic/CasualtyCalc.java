package com.fumbbl.ffb.server.mechanic;

import com.fumbbl.ffb.PlayerState;

/**
 * Pure casualty-roll interpretation extracted from edition-specific RollMechanic classes.
 *
 * BB2016: rolls a 2d6 casualty die; only the first die determines the outcome tier.
 * BB2020/BB2025: rolls a d16; the full value selects the tier.
 * When the tier is Serious Injury, a separate d6 SI roll further determines the specific injury.
 */
public final class CasualtyCalc {

    private CasualtyCalc() {}

    // ── Tier from casualty die ────────────────────────────────────────────────

    /**
     * BB2016: interprets the first die of the 2d6 casualty roll.
     * <ul>
     *   <li>6 → RIP</li>
     *   <li>4–5 → Serious Injury (requires SI table roll)</li>
     *   <li>1–3 → Badly Hurt</li>
     * </ul>
     */
    public static int casualtyTierBB2016(int firstDie) {
        if (firstDie == 6) return PlayerState.RIP;
        if (firstDie >= 4) return PlayerState.SERIOUS_INJURY;
        return PlayerState.BADLY_HURT;
    }

    /**
     * BB2020: interprets a d16 casualty roll (modifiers already applied).
     * <ul>
     *   <li>15+ → RIP</li>
     *   <li>13–14 → Serious Injury (stat reduction, requires SI table roll)</li>
     *   <li>10–12 → Serious Injury (permanent SI)</li>
     *   <li>7–9 → Serious Injury (Seriously Hurt, MNG)</li>
     *   <li>1–6 → Badly Hurt</li>
     * </ul>
     */
    public static int casualtyTierBB2020(int roll) {
        if (roll >= 15) return PlayerState.RIP;
        if (roll >= 7) return PlayerState.SERIOUS_INJURY;
        return PlayerState.BADLY_HURT;
    }

    /**
     * BB2025: interprets a d16 casualty roll (modifiers already applied).
     * <ul>
     *   <li>15+ → RIP</li>
     *   <li>13–14 → Serious Injury (stat reduction, requires SI table roll)</li>
     *   <li>11–12 → Serious Injury (permanent SI)</li>
     *   <li>9–10 → Serious Injury (Seriously Hurt, MNG)</li>
     *   <li>1–8 → Badly Hurt</li>
     * </ul>
     */
    public static int casualtyTierBB2025(int roll) {
        if (roll >= 15) return PlayerState.RIP;
        if (roll >= 9) return PlayerState.SERIOUS_INJURY;
        return PlayerState.BADLY_HURT;
    }

    // ── SI sub-table ──────────────────────────────────────────────────────────

    /**
     * BB2016: returns whether a casualty die first-value triggers the SI detail table.
     * SI detail table is used when first die is 4 or 5.
     */
    public static boolean requiresSIRollBB2016(int firstDie) {
        return firstDie == 4 || firstDie == 5;
    }

    /**
     * BB2020/BB2025: returns whether a d16 casualty roll value triggers the SI detail table.
     * SI detail table is used when the roll is 13 or 14.
     */
    public static boolean requiresSIRollBB2020(int roll) {
        return roll == 13 || roll == 14;
    }

    /**
     * BB2020/BB2025: sub-type for serious injury when not on the SI detail table.
     *
     * @param roll the d16 casualty roll
     * @return SI kind string or {@code null} if not applicable
     */
    public static String seriousInjurySubTypeBB2020(int roll) {
        if (roll >= 10 && roll <= 12) return "SERIOUS_INJURY";
        if (roll >= 7 && roll <= 9) return "SERIOUSLY_HURT";
        return null;
    }

    /**
     * BB2025: sub-type for serious injury when not on the SI detail table.
     */
    public static String seriousInjurySubTypeBB2025(int roll) {
        if (roll >= 11 && roll <= 12) return "SERIOUS_INJURY";
        if (roll >= 9 && roll <= 10) return "SERIOUSLY_HURT";
        return null;
    }
}
