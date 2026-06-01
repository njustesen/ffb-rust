package com.fumbbl.ffb.server.mechanic;

import com.fumbbl.ffb.PlayerState;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

import static org.junit.jupiter.api.Assertions.*;

class CasualtyCalcTest {

    // ══════════════════════════════════════════════════════════════════════════
    // BB2016 — 2d6 casualty die (only first die matters for tier)
    // ══════════════════════════════════════════════════════════════════════════

    @ParameterizedTest(name = "BB2016 firstDie={0} → Badly Hurt")
    @ValueSource(ints = {1, 2, 3})
    void bb2016_firstDie1to3_isBadlyHurt(int die) {
        assertEquals(PlayerState.BADLY_HURT, CasualtyCalc.casualtyTierBB2016(die));
    }

    @ParameterizedTest(name = "BB2016 firstDie={0} → Serious Injury")
    @ValueSource(ints = {4, 5})
    void bb2016_firstDie4or5_isSeriousInjury(int die) {
        assertEquals(PlayerState.SERIOUS_INJURY, CasualtyCalc.casualtyTierBB2016(die));
    }

    @Test
    void bb2016_firstDie6_isRip() {
        assertEquals(PlayerState.RIP, CasualtyCalc.casualtyTierBB2016(6));
    }

    @ParameterizedTest(name = "BB2016 firstDie={0} → requiresSIRoll")
    @ValueSource(ints = {4, 5})
    void bb2016_requiresSIRoll_onlyFor4and5(int die) {
        assertTrue(CasualtyCalc.requiresSIRollBB2016(die));
    }

    @ParameterizedTest(name = "BB2016 firstDie={0} → no SI roll required")
    @ValueSource(ints = {1, 2, 3, 6})
    void bb2016_noSIRoll_for1to3and6(int die) {
        assertFalse(CasualtyCalc.requiresSIRollBB2016(die));
    }

    // ══════════════════════════════════════════════════════════════════════════
    // BB2020 — d16 casualty roll
    // ══════════════════════════════════════════════════════════════════════════

    @ParameterizedTest(name = "BB2020 roll={0} → Badly Hurt")
    @ValueSource(ints = {1, 2, 3, 4, 5, 6})
    void bb2020_roll1to6_isBadlyHurt(int roll) {
        assertEquals(PlayerState.BADLY_HURT, CasualtyCalc.casualtyTierBB2020(roll));
    }

    @ParameterizedTest(name = "BB2020 roll={0} → Serious Injury")
    @ValueSource(ints = {7, 8, 9, 10, 11, 12, 13, 14})
    void bb2020_roll7to14_isSeriousInjury(int roll) {
        assertEquals(PlayerState.SERIOUS_INJURY, CasualtyCalc.casualtyTierBB2020(roll));
    }

    @ParameterizedTest(name = "BB2020 roll={0} → RIP")
    @ValueSource(ints = {15, 16, 17})
    void bb2020_roll15plus_isRip(int roll) {
        assertEquals(PlayerState.RIP, CasualtyCalc.casualtyTierBB2020(roll));
    }

    @Test
    void bb2020_requiresSIRoll_onlyFor13and14() {
        assertFalse(CasualtyCalc.requiresSIRollBB2020(12));
        assertTrue(CasualtyCalc.requiresSIRollBB2020(13));
        assertTrue(CasualtyCalc.requiresSIRollBB2020(14));
        assertFalse(CasualtyCalc.requiresSIRollBB2020(15));
    }

    @Test
    void bb2020_seriousInjurySubType_seriouslyHurt() {
        assertEquals("SERIOUSLY_HURT", CasualtyCalc.seriousInjurySubTypeBB2020(7));
        assertEquals("SERIOUSLY_HURT", CasualtyCalc.seriousInjurySubTypeBB2020(9));
    }

    @Test
    void bb2020_seriousInjurySubType_seriousInjury() {
        assertEquals("SERIOUS_INJURY", CasualtyCalc.seriousInjurySubTypeBB2020(10));
        assertEquals("SERIOUS_INJURY", CasualtyCalc.seriousInjurySubTypeBB2020(12));
    }

    @Test
    void bb2020_seriousInjurySubType_nullForSITableRolls() {
        assertNull(CasualtyCalc.seriousInjurySubTypeBB2020(13));
        assertNull(CasualtyCalc.seriousInjurySubTypeBB2020(14));
    }

    // ══════════════════════════════════════════════════════════════════════════
    // BB2025 — d16 casualty roll (same structure, different thresholds)
    // ══════════════════════════════════════════════════════════════════════════

    @ParameterizedTest(name = "BB2025 roll={0} → Badly Hurt")
    @ValueSource(ints = {1, 2, 3, 4, 5, 6, 7, 8})
    void bb2025_roll1to8_isBadlyHurt(int roll) {
        assertEquals(PlayerState.BADLY_HURT, CasualtyCalc.casualtyTierBB2025(roll));
    }

    @ParameterizedTest(name = "BB2025 roll={0} → Serious Injury")
    @ValueSource(ints = {9, 10, 11, 12, 13, 14})
    void bb2025_roll9to14_isSeriousInjury(int roll) {
        assertEquals(PlayerState.SERIOUS_INJURY, CasualtyCalc.casualtyTierBB2025(roll));
    }

    @ParameterizedTest(name = "BB2025 roll={0} → RIP")
    @ValueSource(ints = {15, 16})
    void bb2025_roll15plus_isRip(int roll) {
        assertEquals(PlayerState.RIP, CasualtyCalc.casualtyTierBB2025(roll));
    }

    @Test
    void bb2025_seriousInjurySubType_seriouslyHurt() {
        assertEquals("SERIOUSLY_HURT", CasualtyCalc.seriousInjurySubTypeBB2025(9));
        assertEquals("SERIOUSLY_HURT", CasualtyCalc.seriousInjurySubTypeBB2025(10));
    }

    @Test
    void bb2025_seriousInjurySubType_seriousInjury() {
        assertEquals("SERIOUS_INJURY", CasualtyCalc.seriousInjurySubTypeBB2025(11));
        assertEquals("SERIOUS_INJURY", CasualtyCalc.seriousInjurySubTypeBB2025(12));
    }

    // ── Cross-edition comparison ──────────────────────────────────────────────

    @Test
    void bb2025_hasHigherBadlyHurtThreshold_than_bb2020() {
        // BB2020: roll 7 → SI; BB2025: roll 7 → Badly Hurt
        assertEquals(PlayerState.SERIOUS_INJURY, CasualtyCalc.casualtyTierBB2020(7));
        assertEquals(PlayerState.BADLY_HURT, CasualtyCalc.casualtyTierBB2025(7));
    }

    @Test
    void bb2025_roll8_isBadlyHurt_unlikeBB2020() {
        assertEquals(PlayerState.SERIOUS_INJURY, CasualtyCalc.casualtyTierBB2020(8));
        assertEquals(PlayerState.BADLY_HURT, CasualtyCalc.casualtyTierBB2025(8));
    }
}
