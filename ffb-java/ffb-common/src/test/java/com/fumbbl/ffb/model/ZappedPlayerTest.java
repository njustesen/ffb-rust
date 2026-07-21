package com.fumbbl.ffb.model;

import com.fumbbl.ffb.PlayerGender;
import com.fumbbl.ffb.PlayerType;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/zapped_player.rs for {@link ZappedPlayer}.
 */
public class ZappedPlayerTest {

	private RosterPlayer playerForTest() {
		RosterPlayer p = new RosterPlayer();
		p.setId("p42");
		p.setName("Bob Blockyhead");
		p.setNr(7);
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

	private ZappedPlayer zap(RosterPlayer player) {
		ZappedPlayer zapped = new ZappedPlayer();
		zapped.init(player, NetCommandTestUtil.gameSource());
		return zapped;
	}

	@Test
	public void getIdDelegatesToOriginal() {
		ZappedPlayer zapped = zap(playerForTest());
		assertEquals("p42", zapped.getId());
	}

	@Test
	public void getNameDelegatesToOriginal() {
		ZappedPlayer zapped = zap(playerForTest());
		assertEquals("Bob Blockyhead", zapped.getName());
	}

	@Test
	public void getNrDelegatesToOriginal() {
		ZappedPlayer zapped = zap(playerForTest());
		assertEquals(7, zapped.getNr());
	}

	@Test
	public void getSkillsReturnsSixZapSkills() {
		ZappedPlayer zapped = zap(playerForTest());
		assertEquals(6, zapped.getSkills().length);
	}

	@Test
	public void getOriginalPlayerIsUnmodified() {
		ZappedPlayer zapped = zap(playerForTest());
		RosterPlayer original = zapped.getOriginalPlayer();
		assertEquals(6, original.getMovement());
		assertEquals(3, original.getStrength());
	}

}
