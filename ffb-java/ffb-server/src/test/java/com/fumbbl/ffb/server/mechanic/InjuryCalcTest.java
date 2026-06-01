package com.fumbbl.ffb.server.mechanic;

import com.fumbbl.ffb.PlayerState;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

import static org.junit.jupiter.api.Assertions.*;

class InjuryCalcTest {

    // ══════════════════════════════════════════════════════════════════════════
    // BB2016
    // ══════════════════════════════════════════════════════════════════════════

    @ParameterizedTest(name = "BB2016 total={0} → Stunned")
    @ValueSource(ints = {2, 3, 4, 5, 6, 7})
    void bb2016_low_totalIsStunned(int total) {
        assertEquals(PlayerState.STUNNED, InjuryCalc.interpretInjuryTotalBB2016(total, false, false));
    }

    @ParameterizedTest(name = "BB2016 total={0} → KO")
    @ValueSource(ints = {8, 9})
    void bb2016_mid_totalIsKO(int total) {
        assertEquals(PlayerState.KNOCKED_OUT, InjuryCalc.interpretInjuryTotalBB2016(total, false, false));
    }

    @ParameterizedTest(name = "BB2016 total={0} → casualty (null)")
    @ValueSource(ints = {10, 11, 12})
    void bb2016_high_totalIsCasualty(int total) {
        assertNull(InjuryCalc.interpretInjuryTotalBB2016(total, false, false));
    }

    @Test
    void bb2016_thickSkull_at8_becomesStunned() {
        assertEquals(PlayerState.STUNNED, InjuryCalc.interpretInjuryTotalBB2016(8, false, true));
    }

    @Test
    void bb2016_thickSkull_at9_stillKO() {
        assertEquals(PlayerState.KNOCKED_OUT, InjuryCalc.interpretInjuryTotalBB2016(9, false, true));
    }

    @Test
    void bb2016_stunty_at7_becomesKO() {
        assertEquals(PlayerState.KNOCKED_OUT, InjuryCalc.interpretInjuryTotalBB2016(7, true, false));
    }

    @Test
    void bb2016_stunty_at9_becomesBadlyHurt() {
        assertEquals(PlayerState.BADLY_HURT, InjuryCalc.interpretInjuryTotalBB2016(9, true, false));
    }

    @Test
    void bb2016_thickSkull_at8_overridesStunty_becauseBB2016ChecksThickSkullFirst() {
        // In BB2016 Thick Skull at 8 is checked BEFORE Stunty — so even a Stunty player with
        // ThickSkull is Stunned at total 8, not KO.
        assertEquals(PlayerState.STUNNED, InjuryCalc.interpretInjuryTotalBB2016(8, true, true));
    }

    @Test
    void bb2016_stuntyThickSkull_at7_isKO_becauseThickSkullOnlyActivatesAt8() {
        // ThickSkull only saves at 8 in BB2016; at 7 with Stunty it's still KO
        assertEquals(PlayerState.KNOCKED_OUT, InjuryCalc.interpretInjuryTotalBB2016(7, true, true));
    }

    // ══════════════════════════════════════════════════════════════════════════
    // BB2020 / BB2025 (share same logic)
    // ══════════════════════════════════════════════════════════════════════════

    @ParameterizedTest(name = "BB2020 total={0} → Stunned")
    @ValueSource(ints = {2, 3, 4, 5, 6, 7})
    void bb2020_low_totalIsStunned(int total) {
        assertEquals(PlayerState.STUNNED, InjuryCalc.interpretInjuryTotalBB2020(total, false, false));
    }

    @ParameterizedTest(name = "BB2020 total={0} → KO")
    @ValueSource(ints = {8, 9})
    void bb2020_mid_totalIsKO(int total) {
        assertEquals(PlayerState.KNOCKED_OUT, InjuryCalc.interpretInjuryTotalBB2020(total, false, false));
    }

    @ParameterizedTest(name = "BB2020 total={0} → casualty (null)")
    @ValueSource(ints = {10, 11, 12})
    void bb2020_high_totalIsCasualty(int total) {
        assertNull(InjuryCalc.interpretInjuryTotalBB2020(total, false, false));
    }

    @Test
    void bb2020_thickSkull_at8_nonStunty_becomesStunned() {
        assertEquals(PlayerState.STUNNED, InjuryCalc.interpretInjuryTotalBB2020(8, false, true));
    }

    @Test
    void bb2020_stunty_at7_becomesKO() {
        assertEquals(PlayerState.KNOCKED_OUT, InjuryCalc.interpretInjuryTotalBB2020(7, true, false));
    }

    @Test
    void bb2020_stunty_at9_becomesBadlyHurt() {
        assertEquals(PlayerState.BADLY_HURT, InjuryCalc.interpretInjuryTotalBB2020(9, true, false));
    }

    @Test
    void bb2020_stuntyThickSkull_at7_thickSkullSaves() {
        // BB2020: Thick Skull overrides Stunty at 7 — Stunned instead of KO
        assertEquals(PlayerState.STUNNED, InjuryCalc.interpretInjuryTotalBB2020(7, true, true));
    }

    @Test
    void bb2020_stuntyThickSkull_at8_thickSkullDoesNotSave() {
        // BB2020: Thick Skull only saves non-Stunty at 8; Stunty+ThickSkull at 8 → KO
        assertEquals(PlayerState.KNOCKED_OUT, InjuryCalc.interpretInjuryTotalBB2020(8, true, true));
    }

    @Test
    void bb2020_stunty_at10_isCasualty() {
        assertNull(InjuryCalc.interpretInjuryTotalBB2020(10, true, false));
    }
}
