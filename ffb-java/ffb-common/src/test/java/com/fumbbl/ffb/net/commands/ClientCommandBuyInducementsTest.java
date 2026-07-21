package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.factory.CardFactory;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.inducement.Card;
import com.fumbbl.ffb.model.InducementSet;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_buy_inducements.rs tests.
 *
 * Java stores real Card and Skill objects (not raw strings), so the populated round-trip uses the
 * game's real "Chop Block" card and "Block" skill obtained from the factories; the wire form of a
 * mercenary skill is Skill.getName() ("Block") rather than the Rust "BLOCK" id.
 */
public class ClientCommandBuyInducementsTest {

	@Test
	public void defaultAvailableGoldIsZero() {
		ClientCommandBuyInducements cmd = new ClientCommandBuyInducements();
		assertEquals(0, cmd.getAvailableGold());
	}

	@Test
	public void storesTeamIdAndGold() {
		ClientCommandBuyInducements cmd =
			new ClientCommandBuyInducements("team_home", 150000, null, null, null, null, null);
		assertEquals("team_home", cmd.getTeamId());
		assertEquals(150000, cmd.getAvailableGold());
	}

	@Test
	public void starPlayerIdsStored() {
		ClientCommandBuyInducements cmd =
			new ClientCommandBuyInducements(null, 0, null, new String[] { "pos1" }, null, null, null);
		assertEquals(1, cmd.getStarPlayerPositionIds().length);
	}

	@Test
	public void getIdIsClientBuyInducements() {
		assertEquals(NetCommandId.CLIENT_BUY_INDUCEMENTS, new ClientCommandBuyInducements().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndTeamId() {
		ClientCommandBuyInducements cmd =
			new ClientCommandBuyInducements("team_home", 0, null, null, null, null, null);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientBuyInducements", json.get("netCommandId").asString());
		assertEquals("team_home", json.get("teamId").asString());
	}

	@Test
	public void roundTripWithPopulatedData() {
		CardFactory cardFactory = NetCommandTestUtil.gameSource().getFactory(Factory.CARD);
		SkillFactory skillFactory = NetCommandTestUtil.gameSource().getFactory(Factory.SKILL);
		Card chopBlock = cardFactory.forName("Chop Block");
		Skill block = skillFactory.forName("Block");
		InducementSet inducementSet = new InducementSet();
		inducementSet.addAvailableCard(chopBlock);
		ClientCommandBuyInducements cmd = new ClientCommandBuyInducements("team_home", 150000, inducementSet,
			new String[] { "pos1" }, new String[] { "merc1" }, new Skill[] { block }, new String[] { "staff1" });
		cmd.setEntropy((byte) 7);
		JsonObject json = cmd.toJsonValue();
		ClientCommandBuyInducements restored =
			new ClientCommandBuyInducements().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 7, restored.getEntropy());
		assertEquals("team_home", restored.getTeamId());
		assertEquals(150000, restored.getAvailableGold());
		assertTrue(restored.getInducementSet().isAvailable(chopBlock));
		assertArrayEquals(new String[] { "pos1" }, restored.getStarPlayerPositionIds());
		assertArrayEquals(new String[] { "merc1" }, restored.getMercenaryPositionIds());
		assertEquals(Arrays.asList("staff1"), restored.getStaffPositionIds());
		assertEquals(1, restored.getMercenarySkills().length);
		assertEquals("Block", restored.getMercenarySkills()[0].getName());
	}

	@Test
	public void roundTripWithDefaultEmptyData() {
		ClientCommandBuyInducements cmd = new ClientCommandBuyInducements();
		JsonObject json = cmd.toJsonValue();
		ClientCommandBuyInducements restored =
			new ClientCommandBuyInducements().initFrom(NetCommandTestUtil.gameSource(), json);
		assertFalse(restored.hasEntropy());
		assertNull(restored.getTeamId());
		assertEquals(0, restored.getAvailableGold());
		assertEquals(0, restored.getInducementSet().getAllCards().length);
		assertEquals(0, restored.getStarPlayerPositionIds().length);
		assertEquals(0, restored.getMercenaryPositionIds().length);
		assertTrue(restored.getStaffPositionIds().isEmpty());
		assertEquals(0, restored.getMercenarySkills().length);
	}
}
