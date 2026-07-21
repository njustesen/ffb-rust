package com.fumbbl.ffb.client.report;

import com.fumbbl.ffb.SpecialEffect;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.ReportSpecialEffectRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class SpellEffectRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player player;

	private void givenPlayer(String id, String name) {
		given(game.getPlayerById(id)).willReturn(player);
		given(player.getName()).willReturn(name);
		given(game.getTeamHome().hasPlayer(player)).willReturn(true);
	}

	@Test
	public void lightningSuccessHitBySpell() {
		givenPlayer("p1", "Zappy");

		ReportSpecialEffectRoll report = new ReportSpecialEffectRoll(SpecialEffect.LIGHTNING, "p1", 4, true);
		List<Run> runs = render(new SpellEffectRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Lightning Spell Effect Roll [ 4 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " is hit by the spell.".equals(r.text)));
	}

	@Test
	public void bombFailureEscapesExplosion() {
		givenPlayer("p1", "Bommy");

		ReportSpecialEffectRoll report = new ReportSpecialEffectRoll(SpecialEffect.BOMB, "p1", 2, false);
		List<Run> runs = render(new SpellEffectRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Bomb Effect Roll [ 2 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " escapes the explosion.".equals(r.text)));
	}

	@Test
	public void bombZeroRollIsAutomaticSuccess() {
		givenPlayer("p1", "Bommy");

		ReportSpecialEffectRoll report = new ReportSpecialEffectRoll(SpecialEffect.BOMB, "p1", 0, true);
		List<Run> runs = render(new SpellEffectRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Bomb Effect Roll [ automatic success ]".equals(r.text)));
	}

	@Test
	public void zapFailureEscapesSpellEffect() {
		givenPlayer("p1", "Zappy");

		ReportSpecialEffectRoll report = new ReportSpecialEffectRoll(SpecialEffect.ZAP, "p1", 1, false);
		List<Run> runs = render(new SpellEffectRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Zap! Spell Effect Roll [ 1 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " escapes the spell effect.".equals(r.text)));
	}

	@Test
	public void fireballSuccessHitByExplosion() {
		givenPlayer("p1", "Fiery");

		ReportSpecialEffectRoll report = new ReportSpecialEffectRoll(SpecialEffect.FIREBALL, "p1", 6, true);
		List<Run> runs = render(new SpellEffectRollMessage(), report);

		assertTrue(runs.stream().anyMatch(r -> "Fireball Spell Effect Roll [ 6 ]".equals(r.text)));
		assertTrue(runs.stream().anyMatch(r -> " is hit by the spell.".equals(r.text)));
	}

	@Test
	public void reportIdIsSpellEffectRoll() {
		assertEquals("spellEffectRoll", new SpellEffectRollMessage().getKey());
	}
}
