package com.fumbbl.ffb.server;

import com.fumbbl.ffb.Weather;
import org.junit.jupiter.api.Test;

import java.util.LinkedHashSet;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/active_effects.rs tests. ActiveEffects is a plain
 * container of per-turn effect state: old weather, skip-restore-weather / stalling flags, the
 * additional-assist team ids, shadowers, and leaders. (The JSON initFrom/toJsonValue round-trip is
 * a separate serialization concern, not part of the Rust unit tests.)
 */
public class ActiveEffectsTest {

	// rust: defaults_are_all_empty_false
	@Test
	public void defaultsAreAllEmptyFalse() {
		ActiveEffects ae = new ActiveEffects();
		assertNull(ae.getOldWeather());
		assertFalse(ae.isSkipRestoreWeather());
		assertFalse(ae.isStalling());
		assertTrue(ae.getTeamIdsAdditionalAssist().isEmpty());
		assertTrue(ae.getShadowers().isEmpty());
		assertTrue(ae.getLeaders().isEmpty());
	}

	// rust: remove_additional_assist_only_removes_matching
	@Test
	public void removeAdditionalAssistOnlyRemovesMatching() {
		ActiveEffects ae = new ActiveEffects();
		Set<String> teams = new LinkedHashSet<>();
		teams.add("team1");
		teams.add("team2");
		ae.setTeamIdsAdditionalAssist(teams);
		ae.removeAdditionalAssist("team1");
		assertEquals(1, ae.getTeamIdsAdditionalAssist().size());
		assertTrue(ae.getTeamIdsAdditionalAssist().contains("team2"));
		assertFalse(ae.getTeamIdsAdditionalAssist().contains("team1"));
	}

	// rust: shadowers_add_and_clear
	@Test
	public void shadowersAddAndClear() {
		ActiveEffects ae = new ActiveEffects();
		ae.addShadower("p1");
		ae.addShadower("p2");
		assertEquals(2, ae.getShadowers().size());
		ae.clearShadowers();
		assertTrue(ae.getShadowers().isEmpty());
	}

	// rust: leaders_add_and_clear
	@Test
	public void leadersAddAndClear() {
		ActiveEffects ae = new ActiveEffects();
		ae.addLeader("coach1");
		assertTrue(ae.getLeaders().contains("coach1"));
		ae.clearLeaders();
		assertTrue(ae.getLeaders().isEmpty());
	}

	// rust: old_weather_set_and_get
	@Test
	public void oldWeatherSetAndGet() {
		ActiveEffects ae = new ActiveEffects();
		ae.setOldWeather(Weather.BLIZZARD);
		assertEquals(Weather.BLIZZARD, ae.getOldWeather());
	}

	// rust: stalling_and_skip_restore_weather_can_be_toggled
	@Test
	public void stallingAndSkipRestoreWeatherCanBeToggled() {
		ActiveEffects ae = new ActiveEffects();
		ae.setStalling(true);
		ae.setSkipRestoreWeather(true);
		assertTrue(ae.isStalling());
		assertTrue(ae.isSkipRestoreWeather());
		ae.setStalling(false);
		ae.setSkipRestoreWeather(false);
		assertFalse(ae.isStalling());
		assertFalse(ae.isSkipRestoreWeather());
	}

	// rust: multiple_leaders_are_tracked
	@Test
	public void multipleLeadersAreTracked() {
		ActiveEffects ae = new ActiveEffects();
		ae.addLeader("coach1");
		ae.addLeader("coach2");
		assertTrue(ae.getLeaders().contains("coach1"));
		assertTrue(ae.getLeaders().contains("coach2"));
		assertEquals(2, ae.getLeaders().size());
	}
}
