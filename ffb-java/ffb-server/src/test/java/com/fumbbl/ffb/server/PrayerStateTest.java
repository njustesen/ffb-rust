package com.fumbbl.ffb.server;

import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.Team;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-model/src/model/prayer_state.rs tests.
 * Rust keys these sets by team/player id strings; Java's server PrayerState keys by Team/Player
 * objects (stored by getId()), so lightweight Team((IFactorySource) null)+setId and RosterPlayer+setId
 * stand in (the ctor does not deref the source).
 */
public class PrayerStateTest {

	private Team team(String id) {
		Team t = new Team((IFactorySource) null);
		t.setId(id);
		return t;
	}

	private RosterPlayer player(String id) {
		RosterPlayer p = new RosterPlayer();
		p.setId(id);
		return p;
	}

	// rust: friends_with_ref_add_check_remove
	@Test
	public void friendsWithRefAddCheckRemove() {
		PrayerState s = new PrayerState();
		Team t1 = team("team1");
		Team t2 = team("team2");
		s.addFriendsWithRef(t1);
		assertTrue(s.isFriendsWithRef(t1));
		assertFalse(s.isFriendsWithRef(t2));
		s.removeFriendsWithRef(t1);
		assertFalse(s.isFriendsWithRef(t1));
	}

	// rust: stallers_add_check_clear
	@Test
	public void stallersAddCheckClear() {
		PrayerState s = new PrayerState();
		RosterPlayer p1 = player("player1");
		RosterPlayer p2 = player("player2");
		s.addStaller(p1);
		s.addStaller(p2);
		assertTrue(s.isStalling(p1));
		assertTrue(s.isStalling(p2));
		s.clearStallers();
		assertFalse(s.isStalling(p1));
		assertFalse(s.isStalling(p2));
	}

	// rust: under_scrutiny_add_remove
	@Test
	public void underScrutinyAddRemove() {
		PrayerState s = new PrayerState();
		Team tA = team("teamA");
		s.addUnderScrutiny(tA);
		assertTrue(s.isUnderScrutiny(tA));
		s.removeUnderScrutiny(tA);
		assertFalse(s.isUnderScrutiny(tA));
	}

	// rust: fouling_frenzy_independent_of_fan_interaction
	@Test
	public void foulingFrenzyIndependentOfFanInteraction() {
		PrayerState s = new PrayerState();
		Team t1 = team("team1");
		Team t2 = team("team2");
		s.addFoulingFrenzy(t1);
		s.addFanInteraction(t2);
		assertTrue(s.hasFoulingFrenzy(t1));
		assertFalse(s.hasFoulingFrenzy(t2));
		assertTrue(s.hasFanInteraction(t2));
		assertFalse(s.hasFanInteraction(t1));
	}

	// rust: additional_spp_teams_returns_ref_to_set
	@Test
	public void additionalSppTeamsReturnsRefToSet() {
		PrayerState s = new PrayerState();
		s.addGetAdditionalCasSpp(team("teamX"));
		s.addGetAdditionalCompletionSpp(team("teamY"));
		assertTrue(s.getAdditionalCasSppTeams().contains("teamX"));
		assertTrue(s.getAdditionalCompletionSppTeams().contains("teamY"));
	}

	// rust: should_not_stall_add_remove
	@Test
	public void shouldNotStallAddRemove() {
		PrayerState s = new PrayerState();
		Team t1 = team("team1");
		s.addShouldNotStall(t1);
		assertTrue(s.shouldNotStall(t1));
		s.removeShouldNotStall(t1);
		assertFalse(s.shouldNotStall(t1));
	}
}
