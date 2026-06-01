package com.fumbbl.ffb.server.mechanic;

/**
 * Documented modifier values for weather effects on rolls.
 *
 * Sign convention (positive = harder for the rolling player, negative = easier):
 * - Dodge/catch/pass: modifier is ADDED to the roll target (min 2).
 *   +1 = harder; -1 = easier.
 * - This matches Java's DodgeModifierCollection/CatchModifierCollection/PassModifierCollection
 *   where tackle zones are +1 (harder) and skill bonuses like Accurate are -1 (easier).
 */
public final class WeatherModifierValues {

    /**
     * Pouring Rain: +1 to catch and pickup targets (harder to handle the ball).
     * Applied to catch and pickup rolls only; not to pass rolls.
     * Source: CatchModifierCollection "Pouring Rain" = +1.
     */
    public static final int POURING_RAIN_CATCH = 1;

    /**
     * Blizzard (BB2016): +0 to pass rolls.
     * Blizzard in BB2016 does not modify the pass roll directly (only movement and catching are affected).
     * Source: BB2016 PassModifierCollection "Blizzard" = 0.
     */
    public static final int BLIZZARD_PASS_BB2016 = 0;

    /**
     * Blizzard (BB2025): +1 to GFI minimum roll (needs 3+ instead of 2+).
     * Source: BB2025 GoForItModifierCollection "Blizzard" = +1.
     */
    public static final int BLIZZARD_GFI_BB2025 = 1;

    /**
     * Very Sunny: +1 to pass rolls (harder to pass in bright sunlight, glare).
     * Source: PassModifierCollection "Very Sunny" = +1.
     */
    public static final int VERY_SUNNY_PASS = 1;

    private WeatherModifierValues() {}
}
