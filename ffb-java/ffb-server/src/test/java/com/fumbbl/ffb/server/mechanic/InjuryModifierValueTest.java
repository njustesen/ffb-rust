package com.fumbbl.ffb.server.mechanic;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Documents the expected modifier values for skills that affect injury rolls.
 *
 * These are used when building an injury roll modifier total:
 *   injuryTotal = d6 + d6 + sum(modifiers)
 * Outcomes: ≤6 → Stunned; 7 → KO (or KO for Stunty); 8-9 → varies; ≥10 → casualty roll.
 */
class InjuryModifierValueTest {

    // ── Mighty Blow (injury application) ─────────────────────────────────────

    @Test
    void mightyBlow_defaultValue_is1() {
        // Both BB2016 (static) and BB2020 (variable, default 1) give +1 to injury
        assertEquals(1, InjuryModifierValues.MIGHTY_BLOW_DEFAULT);
    }

    @Test
    void mightyBlow_cannotApplyBothArmorAndInjury() {
        // Documented constraint: Mighty Blow applies to EITHER armor OR injury, not both.
        // This is enforced by the context check, not the value itself.
        // The test here just documents the rule for future implementors.
        assertTrue(InjuryModifierValues.MIGHTY_BLOW_EXCLUSIVE);
    }

    // ── Dirty Player (injury application) ────────────────────────────────────

    @Test
    void dirtyPlayer_defaultValue_is1() {
        // BB2016: static +1. BB2020: variable, default 1.
        assertEquals(1, InjuryModifierValues.DIRTY_PLAYER_DEFAULT);
    }

    @Test
    void dirtyPlayer_onlyAppliesToFouls() {
        assertTrue(InjuryModifierValues.DIRTY_PLAYER_FOUL_ONLY);
    }

    // ── Niggling Injuries (BB2016) ────────────────────────────────────────────

    @Test
    void nigglingInjury_eachAdds1ToOpponentInjuryRoll() {
        assertEquals(1, InjuryModifierValues.NIGGLING_INJURY_PER_STACK);
    }

    @Test
    void nigglingInjuries_2_give_plus2() {
        assertEquals(2, 2 * InjuryModifierValues.NIGGLING_INJURY_PER_STACK);
    }

    // ── Special Effects ───────────────────────────────────────────────────────

    @Test
    void fireball_injuryModifier_is1() {
        assertEquals(1, InjuryModifierValues.FIREBALL_MODIFIER);
    }

    @Test
    void lightning_injuryModifier_is1() {
        assertEquals(1, InjuryModifierValues.LIGHTNING_MODIFIER);
    }
}
