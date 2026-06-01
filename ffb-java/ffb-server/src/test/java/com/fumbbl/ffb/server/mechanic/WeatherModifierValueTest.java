package com.fumbbl.ffb.server.mechanic;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Documents expected weather modifier values and confirms sign convention.
 *
 * Sign convention: positive = harder for the rolling player (penalty).
 * Matching Java's modifier collections where tackle zones are +1 (penalty) and
 * skills like Accurate/TwoHeads are -1 (benefit).
 */
class WeatherModifierValueTest {

    @Test
    void pouringRain_catchModifier_is_plus1() {
        // Pouring Rain makes catching harder (+1 = harder for catcher)
        assertEquals(1, WeatherModifierValues.POURING_RAIN_CATCH);
    }

    @Test
    void blizzard_passModifier_bb2016_is_0() {
        // BB2016 Blizzard: no penalty to pass rolls (movement effects are separate)
        assertEquals(0, WeatherModifierValues.BLIZZARD_PASS_BB2016);
    }

    @Test
    void blizzard_gfiModifier_bb2025_is_plus1() {
        // BB2025 Blizzard: +1 to GFI = requires 3+ instead of 2+
        assertEquals(1, WeatherModifierValues.BLIZZARD_GFI_BB2025);
    }

    @Test
    void verySunny_passModifier_is_plus1() {
        // Very Sunny: harder to pass (glare) → +1 to pass target
        assertEquals(1, WeatherModifierValues.VERY_SUNNY_PASS);
    }

    @Test
    void signConvention_positiveIsHarder() {
        // All positive values indicate a penalty (harder for roller)
        assertTrue(WeatherModifierValues.POURING_RAIN_CATCH > 0);
        assertTrue(WeatherModifierValues.BLIZZARD_GFI_BB2025 > 0);
        assertTrue(WeatherModifierValues.VERY_SUNNY_PASS > 0);
    }

    @Test
    void gfi_blizzard_bb2025_raisesMinimumFromBase2To3() {
        // GFI base minimum = 2; with Blizzard +1 → minimum = 3
        int gfiBase = 2;
        assertEquals(3, Math.max(2, gfiBase + WeatherModifierValues.BLIZZARD_GFI_BB2025));
    }
}
