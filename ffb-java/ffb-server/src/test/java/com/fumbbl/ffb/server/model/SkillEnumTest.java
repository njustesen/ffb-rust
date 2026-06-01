package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.skill.DeclareCondition;
import com.fumbbl.ffb.model.skill.SkillUsageType;
import com.fumbbl.ffb.PlayerState;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.*;

class SkillEnumTest {

    // ── SkillCategory ───────────────────────────────────────────────────────

    @Test
    void skill_category_count_is_eleven() {
        assertEquals(11, SkillCategory.values().length);
    }

    @ParameterizedTest
    @EnumSource(SkillCategory.class)
    void skill_category_all_have_non_null_name(SkillCategory c) {
        assertNotNull(c.getName());
        assertFalse(c.getName().isEmpty());
    }

    @Test
    void skill_category_general_name() {
        assertEquals("General", SkillCategory.GENERAL.getName());
    }

    @Test
    void skill_category_agility_name() {
        assertEquals("Agility", SkillCategory.AGILITY.getName());
    }

    @Test
    void skill_category_strength_name() {
        assertEquals("Strength", SkillCategory.STRENGTH.getName());
    }

    @Test
    void skill_category_mutation_alt_name_is_mutations() {
        assertEquals("Mutations", SkillCategory.MUTATION.getAltName());
    }

    @Test
    void skill_category_general_alt_name_equals_name() {
        assertEquals(SkillCategory.GENERAL.getName(), SkillCategory.GENERAL.getAltName());
    }

    // ── SkillUsageType ──────────────────────────────────────────────────────

    @Test
    void skill_usage_type_count_is_seven() {
        assertEquals(7, SkillUsageType.values().length);
    }

    @Test
    void skill_usage_type_regular_does_not_track_outside_activation() {
        assertFalse(SkillUsageType.REGULAR.isTrackOutsideActivation());
    }

    @Test
    void skill_usage_type_once_per_turn_tracks_outside_activation() {
        assertTrue(SkillUsageType.ONCE_PER_TURN.isTrackOutsideActivation());
    }

    @Test
    void skill_usage_type_once_per_game_tracks_outside_activation() {
        assertTrue(SkillUsageType.ONCE_PER_GAME.isTrackOutsideActivation());
    }

    @Test
    void skill_usage_type_once_per_turn_removes_effects_at_end_of_turn() {
        assertTrue(SkillUsageType.ONCE_PER_TURN.removeEffectsAtEndOfTurn());
    }

    @Test
    void skill_usage_type_special_does_not_remove_effects_at_end_of_turn() {
        assertFalse(SkillUsageType.SPECIAL.removeEffectsAtEndOfTurn());
    }

    @Test
    void skill_usage_type_once_per_turn_by_team_mate_does_not_remove_effects() {
        assertFalse(SkillUsageType.ONCE_PER_TURN_BY_TEAM_MATE.removeEffectsAtEndOfTurn());
    }

    // ── DeclareCondition ────────────────────────────────────────────────────

    @Test
    void declare_condition_count_is_two() {
        assertEquals(2, DeclareCondition.values().length);
    }

    @Test
    void declare_condition_none_always_fulfilled() {
        assertTrue(DeclareCondition.NONE.fulfilled(null));
        assertTrue(DeclareCondition.NONE.fulfilled(new PlayerState(PlayerState.STANDING)));
    }

    @Test
    void declare_condition_standing_requires_standing_state() {
        assertFalse(DeclareCondition.STANDING.fulfilled(null));
        assertTrue(DeclareCondition.STANDING.fulfilled(new PlayerState(PlayerState.STANDING)));
    }

    @Test
    void declare_condition_standing_fails_for_prone() {
        assertFalse(DeclareCondition.STANDING.fulfilled(new PlayerState(PlayerState.PRONE)));
    }
}
