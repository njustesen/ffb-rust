package com.fumbbl.ffb.model;

import com.eclipsesource.json.JsonValue;
import com.fumbbl.ffb.ApothecaryType;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/turn_data.rs for {@link TurnData}.
 */
public class TurnDataTest {

	private TurnData newTurnData() {
		return new TurnData(null, false);
	}

	@Test
	public void resetClearsFlags() {
		TurnData td = newTurnData();
		td.setBlitzUsed(true);
		td.setPassUsed(true);
		td.startTurn();
		assertFalse(td.isBlitzUsed());
		assertFalse(td.isPassUsed());
	}

	@Test
	public void serdeRoundTrip() {
		TurnData td = newTurnData();
		JsonValue json = td.toJsonValue();
		TurnData back = newTurnData().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(td.getTurnNr(), back.getTurnNr());
		assertEquals(td.getLeaderState(), back.getLeaderState());
	}

	@Test
	public void resetDoesNotClearRerolls() {
		TurnData td = newTurnData();
		td.setReRolls(3);
		td.setApothecaries(1);
		td.setBlitzUsed(true);
		td.startTurn();
		assertEquals(3, td.getReRolls());
		assertEquals(1, td.getApothecaries());
		assertFalse(td.isBlitzUsed());
	}

	@Test
	public void useApothecaryTeamDecrementsTeam() {
		TurnData td = newTurnData();
		td.setApothecaries(2);
		td.useApothecary(ApothecaryType.TEAM);
		assertEquals(1, td.getApothecaries());
		assertEquals(0, td.getWanderingApothecaries());
	}

	@Test
	public void useApothecaryTeamUsesWanderingWhenWanderingGeTeam() {
		TurnData td = newTurnData();
		td.setApothecaries(1);
		td.setWanderingApothecaries(2);
		td.useApothecary(ApothecaryType.TEAM);
		assertEquals(1, td.getWanderingApothecaries());
		assertEquals(0, td.getApothecaries());
	}

	@Test
	public void useApothecaryWanderingAlwaysDecrementsWanderingAndTeam() {
		TurnData td = newTurnData();
		td.setApothecaries(1);
		td.setWanderingApothecaries(1);
		td.useApothecary(ApothecaryType.WANDERING);
		assertEquals(0, td.getWanderingApothecaries());
		assertEquals(0, td.getApothecaries());
	}

	@Test
	public void useApothecaryPlagueDecrementsPlagueDoctorsOnly() {
		TurnData td = newTurnData();
		td.setApothecaries(1);
		td.setPlagueDoctors(2);
		td.useApothecary(ApothecaryType.PLAGUE);
		assertEquals(1, td.getPlagueDoctors());
		assertEquals(1, td.getApothecaries());
	}

	@Test
	public void useApothecaryNoopWhenNoneAvailable() {
		TurnData td = newTurnData();
		td.useApothecary(ApothecaryType.TEAM);
		assertEquals(0, td.getApothecaries());
		assertEquals(0, td.getWanderingApothecaries());
	}

	@Test
	public void allActionFlagsResetTogether() {
		TurnData td = newTurnData();
		td.setBlitzUsed(true);
		td.setFoulUsed(true);
		td.setPassUsed(true);
		td.setHandOverUsed(true);
		td.setTtmUsed(true);
		td.setBombUsed(true);
		td.setSecureTheBallUsed(true);
		td.setPuntUsed(true);
		td.startTurn();
		assertFalse(td.isFoulUsed());
		assertFalse(td.isPassUsed());
		assertFalse(td.isHandOverUsed());
		assertFalse(td.isTtmUsed());
		assertFalse(td.isBombUsed());
		assertFalse(td.isSecureTheBallUsed());
		assertFalse(td.isPuntUsed());
	}

}
