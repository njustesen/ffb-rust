package com.fumbbl.ffb.server.model;

import com.fumbbl.ffb.BlockResult;
import com.fumbbl.ffb.InjuryAttribute;
import com.fumbbl.ffb.PassingDistance;
import com.fumbbl.ffb.Weather;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.EnumSource;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Round-trip and boundary tests for enum types that underlie key game calculations.
 * Mirrors the Rust enum tests in ffb-model/src/enums/.
 */
class EnumRoundTripTest {

    // ── Weather ──────────────────────────────────────────────────────────────

    @Test
    void weather_all_values_have_name() {
        for (Weather w : Weather.values()) {
            assertNotNull(w.getName(), "All Weather values must have a non-null name");
            assertFalse(w.getName().isEmpty(), "All Weather values must have a non-empty name");
        }
    }

    @Test
    void weather_all_values_have_short_name() {
        for (Weather w : Weather.values()) {
            assertNotNull(w.getShortName(), "All Weather values must have a non-null short name");
            assertFalse(w.getShortName().isEmpty(), "All Weather values must have a non-empty short name");
        }
    }

    @ParameterizedTest(name = "Weather.{0} getName() is unique")
    @EnumSource(Weather.class)
    void weather_names_are_unique(Weather w) {
        long count = java.util.Arrays.stream(Weather.values())
            .filter(other -> other.getName().equals(w.getName()))
            .count();
        assertEquals(1, count, "Weather name '" + w.getName() + "' must be unique");
    }

    @ParameterizedTest(name = "Weather.{0} getShortName() is unique")
    @EnumSource(Weather.class)
    void weather_short_names_are_unique(Weather w) {
        long count = java.util.Arrays.stream(Weather.values())
            .filter(other -> other.getShortName().equals(w.getShortName()))
            .count();
        assertEquals(1, count, "Weather shortName '" + w.getShortName() + "' must be unique");
    }

    @Test
    void weather_enum_lookup_by_name() {
        assertEquals(Weather.SWELTERING_HEAT, Weather.valueOf("SWELTERING_HEAT"));
        assertEquals(Weather.VERY_SUNNY, Weather.valueOf("VERY_SUNNY"));
        assertEquals(Weather.NICE, Weather.valueOf("NICE"));
        assertEquals(Weather.POURING_RAIN, Weather.valueOf("POURING_RAIN"));
        assertEquals(Weather.BLIZZARD, Weather.valueOf("BLIZZARD"));
        assertEquals(Weather.INTRO, Weather.valueOf("INTRO"));
    }

    @Test
    void weather_sweltering_heat_name() {
        assertEquals("Sweltering Heat", Weather.SWELTERING_HEAT.getName());
    }

    @Test
    void weather_nice_name() {
        assertEquals("Nice Weather", Weather.NICE.getName());
    }

    @Test
    void weather_blizzard_name() {
        assertEquals("Blizzard", Weather.BLIZZARD.getName());
    }

    @Test
    void weather_pouring_rain_short_name() {
        assertEquals("rain", Weather.POURING_RAIN.getShortName());
    }

    @Test
    void weather_very_sunny_short_name() {
        assertEquals("sunny", Weather.VERY_SUNNY.getShortName());
    }

    // ── BlockResult ──────────────────────────────────────────────────────────

    @Test
    void block_result_all_values_have_name() {
        for (BlockResult br : BlockResult.values()) {
            assertNotNull(br.getName(), "All BlockResult values must have a non-null name");
        }
    }

    @Test
    void block_result_count_is_five() {
        assertEquals(5, BlockResult.values().length, "BlockResult must have exactly 5 values");
    }

    @ParameterizedTest(name = "BlockResult.{0} name is non-empty")
    @EnumSource(BlockResult.class)
    void block_result_name_is_non_empty(BlockResult br) {
        assertFalse(br.getName().isEmpty(), "BlockResult name must not be empty");
    }

    @Test
    void block_result_skull_name() {
        assertEquals("SKULL", BlockResult.SKULL.getName());
    }

    @Test
    void block_result_both_down_name() {
        assertEquals("BOTH DOWN", BlockResult.BOTH_DOWN.getName());
    }

    @Test
    void block_result_pow_name() {
        assertEquals("POW", BlockResult.POW.getName());
    }

    @Test
    void block_result_pushback_name() {
        assertEquals("PUSHBACK", BlockResult.PUSHBACK.getName());
    }

    @Test
    void block_result_pow_pushback_name() {
        assertEquals("POW/PUSH", BlockResult.POW_PUSHBACK.getName());
    }

    // ── PassingDistance ──────────────────────────────────────────────────────

    @Test
    void passing_distance_all_values_have_name() {
        for (PassingDistance pd : PassingDistance.values()) {
            assertNotNull(pd.getName(), "All PassingDistance values must have a non-null name");
        }
    }

    @Test
    void passing_distance_bb2016_quick_pass_is_plus_1() {
        assertEquals(1, PassingDistance.QUICK_PASS.getModifier2016());
    }

    @Test
    void passing_distance_bb2016_short_pass_is_zero() {
        assertEquals(0, PassingDistance.SHORT_PASS.getModifier2016());
    }

    @Test
    void passing_distance_bb2016_long_pass_is_minus_1() {
        assertEquals(-1, PassingDistance.LONG_PASS.getModifier2016());
    }

    @Test
    void passing_distance_bb2016_long_bomb_is_minus_2() {
        assertEquals(-2, PassingDistance.LONG_BOMB.getModifier2016());
    }

    @Test
    void passing_distance_bb2020_quick_pass_is_zero() {
        assertEquals(0, PassingDistance.QUICK_PASS.getModifier2020());
    }

    @Test
    void passing_distance_bb2020_short_pass_is_1() {
        assertEquals(1, PassingDistance.SHORT_PASS.getModifier2020());
    }

    @Test
    void passing_distance_bb2020_long_pass_is_2() {
        assertEquals(2, PassingDistance.LONG_PASS.getModifier2020());
    }

    @Test
    void passing_distance_bb2020_long_bomb_is_3() {
        assertEquals(3, PassingDistance.LONG_BOMB.getModifier2020());
    }

    @Test
    void passing_distance_shortcuts_are_unique() {
        long uniqueShortcuts = java.util.Arrays.stream(PassingDistance.values())
            .map(PassingDistance::getShortcut)
            .distinct().count();
        assertEquals(PassingDistance.values().length, uniqueShortcuts,
            "All PassingDistance shortcuts must be unique");
    }

    @Test
    void passing_distance_quick_pass_shortcut_is_q() {
        assertEquals('Q', PassingDistance.QUICK_PASS.getShortcut());
    }

    @Test
    void passing_distance_long_bomb_shortcut_is_b() {
        assertEquals('B', PassingDistance.LONG_BOMB.getShortcut());
    }

    // ── BB2020 SeriousInjury ──────────────────────────────────────────────────

    @Test
    void bb2020_dead_is_dead() {
        assertTrue(com.fumbbl.ffb.bb2020.SeriousInjury.DEAD.isDead());
    }

    @Test
    void bb2020_seriously_hurt_is_not_dead() {
        assertFalse(com.fumbbl.ffb.bb2020.SeriousInjury.SERIOUSLY_HURT.isDead());
    }

    @Test
    void bb2020_head_injury_reduces_av() {
        assertEquals(InjuryAttribute.AV, com.fumbbl.ffb.bb2020.SeriousInjury.HEAD_INJURY.getInjuryAttribute());
    }

    @Test
    void bb2020_smashed_knee_reduces_ma() {
        assertEquals(InjuryAttribute.MA, com.fumbbl.ffb.bb2020.SeriousInjury.SMASHED_KNEE.getInjuryAttribute());
    }

    @Test
    void bb2020_broken_arm_reduces_pa() {
        assertEquals(InjuryAttribute.PA, com.fumbbl.ffb.bb2020.SeriousInjury.BROKEN_ARM.getInjuryAttribute());
    }

    @Test
    void bb2020_neck_injury_reduces_ag() {
        assertEquals(InjuryAttribute.AG, com.fumbbl.ffb.bb2020.SeriousInjury.NECK_INJURY.getInjuryAttribute());
    }

    @Test
    void bb2020_dislocated_shoulder_reduces_st() {
        assertEquals(InjuryAttribute.ST, com.fumbbl.ffb.bb2020.SeriousInjury.DISLOCATED_SHOULDER.getInjuryAttribute());
    }

    @Test
    void bb2020_seriously_hurt_has_no_injury_attribute() {
        assertNull(com.fumbbl.ffb.bb2020.SeriousInjury.SERIOUSLY_HURT.getInjuryAttribute());
    }

    @Test
    void bb2020_all_serious_injuries_have_name() {
        for (com.fumbbl.ffb.bb2020.SeriousInjury si : com.fumbbl.ffb.bb2020.SeriousInjury.values()) {
            assertNotNull(si.getName(), "All BB2020 SeriousInjury values must have a name");
        }
    }

    // ── BB2016 SeriousInjury ──────────────────────────────────────────────────

    @Test
    void bb2016_dead_is_dead() {
        assertTrue(com.fumbbl.ffb.bb2016.SeriousInjury.DEAD.isDead());
    }

    @Test
    void bb2016_broken_ribs_is_mng_not_dead() {
        assertFalse(com.fumbbl.ffb.bb2016.SeriousInjury.BROKEN_RIBS.isDead());
        assertNull(com.fumbbl.ffb.bb2016.SeriousInjury.BROKEN_RIBS.getInjuryAttribute());
    }

    @Test
    void bb2016_smashed_knee_is_niggling_injury() {
        assertEquals(InjuryAttribute.NI, com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE.getInjuryAttribute());
    }

    @Test
    void bb2016_broken_neck_reduces_ag() {
        assertEquals(InjuryAttribute.AG, com.fumbbl.ffb.bb2016.SeriousInjury.BROKEN_NECK.getInjuryAttribute());
    }

    @Test
    void bb2016_smashed_collar_bone_reduces_st() {
        assertEquals(InjuryAttribute.ST, com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_COLLAR_BONE.getInjuryAttribute());
    }

    @Test
    void bb2016_smashed_hip_reduces_ma() {
        assertEquals(InjuryAttribute.MA, com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_HIP.getInjuryAttribute());
    }

    @Test
    void bb2016_all_serious_injuries_have_name() {
        for (com.fumbbl.ffb.bb2016.SeriousInjury si : com.fumbbl.ffb.bb2016.SeriousInjury.values()) {
            assertNotNull(si.getName(), "All BB2016 SeriousInjury values must have a name");
        }
    }
}
