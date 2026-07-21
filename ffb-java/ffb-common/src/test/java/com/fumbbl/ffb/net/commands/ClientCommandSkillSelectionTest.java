package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_skill_selection.rs tests.
 *
 * The Rust struct simplifies {@code skill} to a plain identifier string; the Java
 * class stores a real {@link Skill} object serialized by its name. The tests resolve
 * concrete skills via the {@code SKILL} factory and assert against the skill name.
 * Java's {@code getId()} intentionally keeps the legacy {@code CLIENT_PRAYER_SELECTION}
 * wire id.
 */
public class ClientCommandSkillSelectionTest {

	private Skill skill(String name) {
		SkillFactory factory = NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL);
		return factory.forName(name);
	}

	@Test
	public void defaultHasNoPlayerOrSkill() {
		ClientCommandSkillSelection cmd = new ClientCommandSkillSelection();
		assertNull(cmd.getPlayerId());
		assertNull(cmd.getSkill());
	}

	@Test
	public void storesPlayerIdAndSkillId() {
		Skill block = skill("Block");
		ClientCommandSkillSelection cmd = new ClientCommandSkillSelection("player_2", block);
		assertEquals("player_2", cmd.getPlayerId());
		assertEquals(block, cmd.getSkill());
	}

	@Test
	public void getIdIsClientPrayerSelection() {
		assertEquals(NetCommandId.CLIENT_PRAYER_SELECTION, new ClientCommandSkillSelection().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndSkillKey() {
		Skill block = skill("Block");
		ClientCommandSkillSelection cmd = new ClientCommandSkillSelection("player_2", block);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientPrayerSelection", json.get("netCommandId").asString());
		assertEquals(block.getName(), json.get("skill").asString());
	}

	@Test
	public void roundTripWithData() {
		Skill dodge = skill("Dodge");
		ClientCommandSkillSelection cmd = new ClientCommandSkillSelection("player_3", dodge);
		cmd.setEntropy((byte) 4);
		JsonObject json = cmd.toJsonValue();
		ClientCommandSkillSelection restored = new ClientCommandSkillSelection().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals("player_3", restored.getPlayerId());
		assertEquals(dodge, restored.getSkill());
		assertEquals((byte) 4, restored.getEntropy());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandSkillSelection cmd = new ClientCommandSkillSelection();
		JsonObject json = cmd.toJsonValue();
		ClientCommandSkillSelection restored = new ClientCommandSkillSelection().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getPlayerId());
		assertNull(restored.getSkill());
		assertFalse(restored.hasEntropy());
	}
}
