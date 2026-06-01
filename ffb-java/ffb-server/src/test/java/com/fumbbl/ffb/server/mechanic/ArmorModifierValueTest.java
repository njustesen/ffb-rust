package com.fumbbl.ffb.server.mechanic;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Documents expected armor modifier values and armor-break thresholds.
 *
 * See also RollCalcTest for isArmourBroken() and applyFixedArmourReduction() behavior.
 */
class ArmorModifierValueTest {

    @Test
    void mightyBlow_armorModifier_is1() {
        assertEquals(1, ArmorModifierValues.MIGHTY_BLOW_DEFAULT);
    }

    @Test
    void dirtyPlayer_armorModifier_is1() {
        assertEquals(1, ArmorModifierValues.DIRTY_PLAYER_DEFAULT);
    }

    @Test
    void pilingOn_is2() {
        assertEquals(2, ArmorModifierValues.PILING_ON);
    }

    @Test
    void stunty_is_minus1() {
        assertEquals(-1, ArmorModifierValues.STUNTY);
    }

    @Test
    void fixedArmourCap_bb2016_is7() {
        assertEquals(7, ArmorModifierValues.FIXED_ARMOUR_CAP_BB2016);
    }

    @Test
    void fixedArmourCap_bb2020_is8() {
        assertEquals(8, ArmorModifierValues.FIXED_ARMOUR_CAP_BB2020);
    }

    @Test
    void bb2020_cap_higher_than_bb2016() {
        assertTrue(ArmorModifierValues.FIXED_ARMOUR_CAP_BB2020 > ArmorModifierValues.FIXED_ARMOUR_CAP_BB2016);
    }
}
