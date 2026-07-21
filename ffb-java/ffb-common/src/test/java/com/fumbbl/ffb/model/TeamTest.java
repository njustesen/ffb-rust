package com.fumbbl.ffb.model;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.PlayerType;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/team.rs for {@link Team}.
 */
public class TeamTest {

	private Team emptyTeam() {
		Team t = new Team(NetCommandTestUtil.gameSource());
		t.setId("t1");
		t.setName("Humans");
		t.setRace("Human");
		t.setRosterId("human");
		t.setCoach("Coach");
		t.setReRolls(3);
		t.setApothecaries(1);
		return t;
	}

	private RosterPlayer player(String id, int nr) {
		RosterPlayer p = new RosterPlayer();
		p.setId(id);
		p.setName("Joe");
		p.setNr(nr);
		p.setPositionId("lineman");
		p.setType(PlayerType.REGULAR);
		p.setGender(PlayerGender.MALE);
		p.setMovement(6);
		p.setStrength(3);
		p.setAgility(3);
		p.setPassing(4);
		p.setArmour(8);
		return p;
	}

	@Test
	public void serdeRoundTrip() {
		Team t = emptyTeam();
		JsonValue json = t.toJsonValue();
		Team back = new Team(NetCommandTestUtil.gameSource()).initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("t1", back.getId());
		assertEquals(3, back.getReRolls());
	}

	@Test
	public void playerLookup() {
		Team t = emptyTeam();
		RosterPlayer p = player("p1", 1);
		t.addPlayer(p);
		assertNotNull(t.getPlayerById("p1"));
		assertNull(t.getPlayerById("p2"));
		assertTrue(t.hasPlayer(p));
	}

	@Test
	public void playerByNrFindsCorrectPlayer() {
		Team t = emptyTeam();
		t.addPlayer(player("p1", 5));
		assertEquals("p1", t.getPlayerByNr(5).getId());
		assertNull(t.getPlayerByNr(99));
	}

	@Test
	public void playerMutModifiesPlayerInPlace() {
		Team t = emptyTeam();
		RosterPlayer p = player("p1", 1);
		t.addPlayer(p);
		p.setCurrentSpps(10);
		assertEquals(10, t.getPlayerById("p1").getCurrentSpps());
	}

	@Test
	public void hasPlayerFalseForUnknownId() {
		Team t = emptyTeam();
		assertNull(t.getPlayerById("nobody"));
	}

	@Test
	public void multiplePlayersLookedUpIndependently() {
		Team t = emptyTeam();
		for (int nr = 1; nr <= 3; nr++) {
			t.addPlayer(player("p" + nr, nr));
		}
		assertEquals(3, t.getPlayers().length);
		assertEquals(2, t.getPlayerById("p2").getNr());
		assertNull(t.getPlayerById("p4"));
	}

	@Test
	public void playersListReflectsAddedPlayers() {
		Team t = emptyTeam();
		assertEquals(0, t.getPlayers().length);
		t.addPlayer(player("p1", 1));
		assertEquals(1, t.getPlayers().length);
	}

}
