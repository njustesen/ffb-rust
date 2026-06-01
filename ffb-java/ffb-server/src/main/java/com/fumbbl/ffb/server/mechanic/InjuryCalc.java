package com.fumbbl.ffb.server.mechanic;

import com.fumbbl.ffb.PlayerState;

/**
 * Pure injury-roll interpretation extracted from edition-specific RollMechanic classes.
 *
 * All editions share the same base table; Stunty and Thick Skull interactions differ
 * between BB2016 and BB2020/BB2025.
 */
public final class InjuryCalc {

    private InjuryCalc() {}

    /**
     * Interprets an injury roll total for BB2016 rules.
     *
     * <ul>
     *   <li>8 + Thick Skull → Stunned (ThickSkull is checked first in BB2016)</li>
     *   <li>7 + Stunty → KO</li>
     *   <li>9 + Stunty → Badly Hurt</li>
     *   <li>10+ → casualty (returns {@code null})</li>
     *   <li>8–9 → KO</li>
     *   <li>2–7 → Stunned</li>
     * </ul>
     *
     * @return PlayerState constant, or {@code null} when a casualty roll is required
     */
    public static Integer interpretInjuryTotalBB2016(int total, boolean isStunty, boolean hasThickSkull) {
        if (total == 8 && hasThickSkull) return PlayerState.STUNNED;
        if (total == 7 && isStunty) return PlayerState.KNOCKED_OUT;
        if (total == 9 && isStunty) return PlayerState.BADLY_HURT;
        if (total > 9) return null;
        if (total > 7) return PlayerState.KNOCKED_OUT;
        return PlayerState.STUNNED;
    }

    /**
     * Interprets an injury roll total for BB2020/BB2025 rules.
     *
     * <ul>
     *   <li>7 + Stunty + Thick Skull → Stunned (Thick Skull saves even Stunty players at 7)</li>
     *   <li>7 + Stunty → KO</li>
     *   <li>8 + Thick Skull (non-Stunty) → Stunned</li>
     *   <li>9 + Stunty → Badly Hurt</li>
     *   <li>10+ → casualty (returns {@code null})</li>
     *   <li>8–9 → KO</li>
     *   <li>2–7 → Stunned</li>
     * </ul>
     */
    public static Integer interpretInjuryTotalBB2020(int total, boolean isStunty, boolean hasThickSkull) {
        if (total == 7 && isStunty) {
            return hasThickSkull ? PlayerState.STUNNED : PlayerState.KNOCKED_OUT;
        }
        if (total == 8 && hasThickSkull && !isStunty) return PlayerState.STUNNED;
        if (total == 9 && isStunty) return PlayerState.BADLY_HURT;
        if (total > 9) return null;
        if (total > 7) return PlayerState.KNOCKED_OUT;
        return PlayerState.STUNNED;
    }
}
