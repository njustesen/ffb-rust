package com.fumbbl.ffb.net.commands;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.SpecialEffect;
import com.fumbbl.ffb.net.NetCommandId;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonObject;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-protocol/src/commands/client_command_wizard_spell.rs tests.
 */
public class ClientCommandWizardSpellTest {

	@Test
	public void defaultNone() {
		ClientCommandWizardSpell cmd = new ClientCommandWizardSpell();
		assertNull(cmd.getWizardSpell());
	}

	@Test
	public void withSpellStoresValues() {
		ClientCommandWizardSpell cmd = new ClientCommandWizardSpell(SpecialEffect.FIREBALL, new FieldCoordinate(3, 5));
		assertNotNull(cmd.getWizardSpell());
		assertEquals(new FieldCoordinate(3, 5), cmd.getTargetCoordinate());
	}

	@Test
	public void getIdIsClientWizardSpell() {
		assertEquals(NetCommandId.CLIENT_WIZARD_SPELL, new ClientCommandWizardSpell().getId());
	}

	@Test
	public void toJsonValueHasNetCommandIdAndWizardSpell() {
		ClientCommandWizardSpell cmd = new ClientCommandWizardSpell(SpecialEffect.ZAP, new FieldCoordinate(1, 1));
		JsonObject json = cmd.toJsonValue();
		assertEquals("clientWizardSpell", json.get("netCommandId").asString());
		assertEquals("zap", json.get("wizardSpell").asString());
	}

	@Test
	public void roundTripWithSpellAndEntropy() {
		ClientCommandWizardSpell cmd = new ClientCommandWizardSpell(SpecialEffect.BOMB, new FieldCoordinate(4, 6));
		cmd.setEntropy((byte) 3);
		JsonObject json = cmd.toJsonValue();
		ClientCommandWizardSpell restored = new ClientCommandWizardSpell()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertTrue(restored.hasEntropy());
		assertEquals((byte) 3, restored.getEntropy());
		assertEquals(SpecialEffect.BOMB, restored.getWizardSpell());
		assertEquals(new FieldCoordinate(4, 6), restored.getTargetCoordinate());
	}

	@Test
	public void roundTripWithNoSpell() {
		ClientCommandWizardSpell cmd = new ClientCommandWizardSpell();
		JsonObject json = cmd.toJsonValue();
		ClientCommandWizardSpell restored = new ClientCommandWizardSpell()
			.initFrom(NetCommandTestUtil.gameSource(), json);
		assertNull(restored.getWizardSpell());
		assertNull(restored.getTargetCoordinate());
	}
}
