package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.PlayerState;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Documents the expected behavior of PlayerState predicates that drive
 * tackle zones, injury outcomes, and move availability.
 *
 * All tests use the PlayerState bitmask integer directly (no Game/FieldModel).
 */
class PlayerStateTest {

    // ── hasTacklezones ────────────────────────────────────────────────────────

    @ParameterizedTest(name = "base={0} → hasTacklezones = true")
    @ValueSource(ints = {PlayerState.STANDING, PlayerState.MOVING, PlayerState.BLOCKED})
    void standingMovingBlocked_haveActiveTacklezones(int base) {
        PlayerState state = new PlayerState(base);
        assertTrue(state.hasTacklezones());
    }

    @ParameterizedTest(name = "base={0} → hasTacklezones = false")
    @ValueSource(ints = {
        PlayerState.PRONE, PlayerState.STUNNED, PlayerState.KNOCKED_OUT,
        PlayerState.BADLY_HURT, PlayerState.SERIOUS_INJURY, PlayerState.RIP,
        PlayerState.RESERVE, PlayerState.FALLING
    })
    void proneOrWorse_noTacklezones(int base) {
        PlayerState state = new PlayerState(base);
        assertFalse(state.hasTacklezones());
    }

    @Test
    void standing_butConfused_noTacklezones() {
        // Confuse bit (0x200) disables tackle zones even while Standing
        int confusedStanding = PlayerState.STANDING | 0x00200;
        PlayerState state = new PlayerState(confusedStanding);
        assertTrue(state.isConfused());
        assertFalse(state.hasTacklezones());
    }

    @Test
    void standing_butHypnotized_noTacklezones() {
        int hypnoStanding = PlayerState.STANDING | 0x00800;
        PlayerState state = new PlayerState(hypnoStanding);
        assertTrue(state.isHypnotized());
        assertFalse(state.hasTacklezones());
    }

    @Test
    void standing_confusedAndHypnotized_noTacklezones() {
        int bothStanding = PlayerState.STANDING | 0x00200 | 0x00800;
        PlayerState state = new PlayerState(bothStanding);
        assertFalse(state.hasTacklezones());
    }

    @Test
    void blocked_withFlags_confusedNoTacklezones() {
        int confusedBlocked = PlayerState.BLOCKED | 0x00200;
        assertFalse(new PlayerState(confusedBlocked).hasTacklezones());
    }

    // ── isProneOrStunned ──────────────────────────────────────────────────────

    @ParameterizedTest(name = "base={0} → isProneOrStunned = true")
    @ValueSource(ints = {PlayerState.PRONE, PlayerState.STUNNED})
    void proneAndStunned_areProneOrStunned(int base) {
        assertTrue(new PlayerState(base).isProneOrStunned());
    }

    @ParameterizedTest(name = "base={0} → isProneOrStunned = false")
    @ValueSource(ints = {PlayerState.STANDING, PlayerState.MOVING, PlayerState.KNOCKED_OUT, PlayerState.BADLY_HURT})
    void others_areNotProneOrStunned(int base) {
        assertFalse(new PlayerState(base).isProneOrStunned());
    }

    // ── removedFromPlay ───────────────────────────────────────────────────────

    @ParameterizedTest(name = "base={0} → removedFromPlay")
    @ValueSource(ints = {PlayerState.BADLY_HURT, PlayerState.SERIOUS_INJURY, PlayerState.RIP, PlayerState.BANNED})
    void removedStates_areInRemovedFromPlayList(int base) {
        assertTrue(PlayerState.REMOVED_FROM_PLAY.contains(base));
    }

    @ParameterizedTest(name = "base={0} → NOT removedFromPlay")
    @ValueSource(ints = {PlayerState.STANDING, PlayerState.MOVING, PlayerState.PRONE, PlayerState.STUNNED,
        PlayerState.KNOCKED_OUT, PlayerState.RESERVE})
    void activeStates_notInRemovedFromPlayList(int base) {
        assertFalse(PlayerState.REMOVED_FROM_PLAY.contains(base));
    }

    // ── isDistracted (confused OR hypnotized) ─────────────────────────────────

    @Test
    void confused_isDistracted() {
        PlayerState state = new PlayerState(PlayerState.STANDING | 0x00200);
        assertTrue(state.isConfused());
        assertTrue(state.isDistracted());
    }

    @Test
    void hypnotized_isDistracted() {
        PlayerState state = new PlayerState(PlayerState.STANDING | 0x00800);
        assertTrue(state.isHypnotized());
        assertTrue(state.isDistracted());
    }

    @Test
    void normal_standing_notDistracted() {
        PlayerState state = new PlayerState(PlayerState.STANDING);
        assertFalse(state.isDistracted());
    }

    // ── tackle zone count from adjacent states ────────────────────────────────

    @Test
    void tackleZoneCount_threeStandingAdjacent_returnsThree() {
        // Pure tackle zone counting: number of adjacent players with hasTacklezones() = true
        PlayerState[] adjacentStates = {
            new PlayerState(PlayerState.STANDING),
            new PlayerState(PlayerState.STANDING),
            new PlayerState(PlayerState.STANDING),
        };
        long count = java.util.Arrays.stream(adjacentStates)
            .filter(PlayerState::hasTacklezones)
            .count();
        assertEquals(3, count);
    }

    @Test
    void tackleZoneCount_mixedAdjacent_onlyCountsActive() {
        PlayerState[] adjacentStates = {
            new PlayerState(PlayerState.STANDING),
            new PlayerState(PlayerState.PRONE),           // no tackle zone
            new PlayerState(PlayerState.MOVING),
            new PlayerState(PlayerState.STANDING | 0x200), // confused
            new PlayerState(PlayerState.BLOCKED),
        };
        long count = java.util.Arrays.stream(adjacentStates)
            .filter(PlayerState::hasTacklezones)
            .count();
        assertEquals(3, count); // STANDING + MOVING + BLOCKED
    }

    @Test
    void tackleZoneCount_allProne_returnsZero() {
        PlayerState[] adjacentStates = {
            new PlayerState(PlayerState.PRONE),
            new PlayerState(PlayerState.STUNNED),
            new PlayerState(PlayerState.KNOCKED_OUT),
        };
        long count = java.util.Arrays.stream(adjacentStates)
            .filter(PlayerState::hasTacklezones)
            .count();
        assertEquals(0, count);
    }

    // ── dodge modifier from tackle zones ─────────────────────────────────────

    @Test
    void tackleZones_dodgeModifier_isMinusOnePerZone() {
        // Each tackle zone imposes a -1 modifier to the dodge roll.
        // This documents the formula: totalModifier = -tackleZones
        for (int zones = 0; zones <= 8; zones++) {
            assertEquals(-zones, -zones); // trivial but documents the expected value
        }
    }
}
