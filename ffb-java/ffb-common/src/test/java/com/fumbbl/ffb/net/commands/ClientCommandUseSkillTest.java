package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.ReRolledAction;
import com.fumbbl.ffb.factory.ReRolledActionFactory;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_use_skill.rs tests.
 * The Rust port stores skill/reRolledAction as name strings; Java stores typed
 * {@link Skill} and {@link ReRolledAction} objects obtained from the game-context factories.
 */
public class ClientCommandUseSkillTest {

	private static Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	private static ReRolledAction action(String name) {
		ReRolledActionFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.RE_ROLLED_ACTION);
		return factory.forName(name);
	}

	@Test
	public void skillUsedFlag() {
		ClientCommandUseSkill cmd = new ClientCommandUseSkill(null, true, null, null, false);
		assertTrue(cmd.isSkillUsed());
	}

	@Test
	public void defaultAllFalse() {
		ClientCommandUseSkill cmd = new ClientCommandUseSkill();
		assertFalse(cmd.isSkillUsed());
		assertFalse(cmd.isNeverUse());
	}

	@Test
	public void getIdIsClientUseSkill() {
		assertEquals(NetCommandId.CLIENT_USE_SKILL, new ClientCommandUseSkill().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndSkillUsed() {
		ClientCommandUseSkill cmd = new ClientCommandUseSkill(null, true, null, null, false);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientUseSkill", json.get("netCommandId").asString());
		assertTrue(json.get("skillUsed").asBoolean());
	}

	@Test
	public void roundTripWithAllFieldsAndEntropy() {
		Skill dodge = skill("Dodge");
		ReRolledAction dodgeAction = action("Dodge");
		assertNotNull(dodge);
		assertNotNull(dodgeAction);
		ClientCommandUseSkill cmd = new ClientCommandUseSkill(dodge, true, "p1", dodgeAction, true);
		cmd.setEntropy((byte) 5);
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseSkill restored = new ClientCommandUseSkill()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 5, restored.getEntropy());
		assertNotNull(restored.getSkill());
		assertEquals(dodge.getName(), restored.getSkill().getName());
		assertTrue(restored.isSkillUsed());
		assertTrue(restored.isNeverUse());
		assertEquals("p1", restored.getPlayerId());
		assertNotNull(restored.getReRolledAction());
		assertEquals(dodgeAction.getName(), restored.getReRolledAction().getName());
	}

	@Test
	public void roundTripWithNoOptionalFields() {
		ClientCommandUseSkill cmd = new ClientCommandUseSkill();
		JsonObject json = cmd.toJsonValue();
		ClientCommandUseSkill restored = new ClientCommandUseSkill()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getSkill());
		assertFalse(restored.isSkillUsed());
		assertFalse(restored.isNeverUse());
		assertNull(restored.getPlayerId());
		assertNull(restored.getReRolledAction());
	}
}
