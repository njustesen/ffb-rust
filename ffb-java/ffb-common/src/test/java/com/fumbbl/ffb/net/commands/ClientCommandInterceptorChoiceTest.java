package com.fumbbl.ffb.net.commands;

import com.eclipsesource.json.JsonObject;
import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.factory.SkillFactory;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_interceptor_choice.rs tests.
 *
 * The Rust struct stores the interception skill as a plain name string; the Java class stores a
 * real {@link Skill} object obtained from the skill factory. Assertions are adapted accordingly.
 */
public class ClientCommandInterceptorChoiceTest {

	@Test
	public void interceptorIdStored() {
		ClientCommandInterceptorChoice cmd = new ClientCommandInterceptorChoice("p5", null);
		assertEquals("p5", cmd.getInterceptorId());
		assertNull(cmd.getInterceptionSkill());
	}

	@Test
	public void defaultBothNone() {
		ClientCommandInterceptorChoice cmd = new ClientCommandInterceptorChoice();
		assertNull(cmd.getInterceptorId());
	}

	@Test
	public void getIdIsClientInterceptorChoice() {
		assertEquals(NetCommandId.CLIENT_INTERCEPTOR_CHOICE, new ClientCommandInterceptorChoice().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndInterceptorId() {
		ClientCommandInterceptorChoice cmd = new ClientCommandInterceptorChoice("p5", null);
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientInterceptorChoice", json.get("netCommandId").asString());
		assertEquals("p5", json.get("interceptorId").asString());
	}

	@Test
	public void roundTripWithSkillAndEntropy() {
		SkillFactory skillFactory = NetCommandTestUtil.gameSource().getFactory(Factory.SKILL);
		Skill dodge = skillFactory.forName("Dodge");
		ClientCommandInterceptorChoice cmd = new ClientCommandInterceptorChoice("p5", dodge);
		cmd.setEntropy((byte) 8);
		JsonObject json = cmd.toJsonValue();
		ClientCommandInterceptorChoice restored = new ClientCommandInterceptorChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals((byte) 8, restored.getEntropy());
		assertEquals("p5", restored.getInterceptorId());
		assertEquals("Dodge", restored.getInterceptionSkill().getName());
	}

	@Test
	public void roundTripDefault() {
		ClientCommandInterceptorChoice cmd = new ClientCommandInterceptorChoice();
		JsonObject json = cmd.toJsonValue();
		ClientCommandInterceptorChoice restored = new ClientCommandInterceptorChoice().initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getInterceptorId());
		assertNull(restored.getInterceptionSkill());
	}
}
