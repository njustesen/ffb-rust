package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_lightning.rs tests.
 * The Lightning +1 applies to the armour roll only when needed; otherwise to the injury roll —
 * never both.
 */
public class InjuryTypeLightningTest {

	private GameState gameState;
	private Game game;
	private IStep step;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3);
		game = gameState.getGame();
		step = GameFixture.createStep(gameState, StepId.INIT_START_GAME);
		GameFixture.placePlayer(gameState, "home1", 2, 2); // attacker
		GameFixture.placePlayer(gameState, "away1", 5, 5); // defender
	}

	private RosterPlayer attacker() {
		return (RosterPlayer) game.getPlayerById("home1");
	}

	private RosterPlayer defender() {
		return (RosterPlayer) game.getPlayerById("away1");
	}

	private void handleInjury(InjuryTypeLightning lightning) {
		lightning.injuryContext().setAttackerId("home1");
		lightning.injuryContext().setDefenderId("away1");
		lightning.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasArmorModifier(InjuryTypeLightning lightning, String name) {
		return Arrays.stream(lightning.injuryContext().getArmorModifiers())
			.anyMatch(m -> name.equals(m.getName()));
	}

	private boolean hasInjuryModifier(InjuryTypeLightning lightning, String name) {
		return Arrays.stream(lightning.injuryContext().getInjuryModifiers())
			.anyMatch(m -> name.equals(m.getName()));
	}

	// rust: armor_save_results_in_prone
	@Test
	public void armorSaveResultsInProne() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1); // 2 (+1 lightning) < 13 -> save
		InjuryTypeLightning lightning = new InjuryTypeLightning();
		handleInjury(lightning);
		assertEquals(PlayerState.PRONE, lightning.injuryContext().getInjury().getBase());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeLightning lightning = new InjuryTypeLightning();
		handleInjury(lightning);
		assertTrue(lightning.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, lightning.injuryContext().getInjury().getBase());
	}

	// rust: turnover_default_true (bug #7: Lightning does not override fallingDownCausesTurnover)
	@Test
	public void turnoverDefaultTrue() {
		assertTrue(new InjuryTypeLightning().injuryType().fallingDownCausesTurnover());
	}

	// rust: send_to_box_reason_is_lightning
	@Test
	public void sendToBoxReasonIsLightning() {
		assertEquals(SendToBoxReason.LIGHTNING, new InjuryTypeLightning().sendToBoxReason());
	}

	// rust: mighty_blow_adds_injury_modifier
	@Test
	public void mightyBlowAddsInjuryModifier() {
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeLightning lightning = new InjuryTypeLightning();
		handleInjury(lightning);
		assertTrue(lightning.injuryContext().isArmorBroken());
		assertTrue(hasInjuryModifier(lightning, "Mighty Blow"));
	}

	// rust: no_mighty_blow_no_injury_modifier
	@Test
	public void noMightyBlowNoInjuryModifier() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeLightning lightning = new InjuryTypeLightning();
		handleInjury(lightning);
		assertFalse(hasInjuryModifier(lightning, "Mighty Blow"));
	}

	// rust: lightning_bonus_needed_for_armor_break_excludes_injury_bonus
	@Test
	public void lightningBonusNeededForArmorBreakExcludesInjuryBonus() {
		defender().setArmour(7);
		GameFixture.installScriptedDice(gameState, 3, 3, 1, 1); // 6 < 7; +1 lightning -> 7 >= 7 break
		InjuryTypeLightning lightning = new InjuryTypeLightning();
		handleInjury(lightning);
		assertTrue(hasArmorModifier(lightning, "Lightning"),
			"Lightning armor modifier should have been needed and applied");
		assertTrue(lightning.injuryContext().isArmorBroken());
		assertFalse(hasInjuryModifier(lightning, "Lightning"),
			"the bonus consumed by the armor roll must not also boost the injury roll");
	}

	// rust: armor_breaks_naturally_lightning_bonus_applies_to_injury_instead
	@Test
	public void armorBreaksNaturallyLightningBonusAppliesToInjuryInstead() {
		defender().setArmour(6);
		GameFixture.installScriptedDice(gameState, 3, 3, 1, 1); // 6 >= 6 -> natural break
		InjuryTypeLightning lightning = new InjuryTypeLightning();
		handleInjury(lightning);
		assertTrue(lightning.injuryContext().isArmorBroken());
		assertFalse(hasArmorModifier(lightning, "Lightning"),
			"Lightning armor modifier must not be added when the raw roll already broke armor");
		assertTrue(hasInjuryModifier(lightning, "Lightning"),
			"the unused bonus applies to the injury roll instead");
	}

	// rust: defender_ignoring_skill_armor_modifiers_never_gets_lightning_armor_bonus
	@Test
	public void defenderIgnoringSkillArmorModifiersNeverGetsLightningArmorBonus() {
		defender().setArmour(7);
		defender().addSkill(GameFixture.skill(game, "Iron Hard Skin"));
		GameFixture.installScriptedDice(gameState, 3, 3); // 6 < 7, bonus denied -> stays unbroken
		InjuryTypeLightning lightning = new InjuryTypeLightning();
		handleInjury(lightning);
		assertFalse(hasArmorModifier(lightning, "Lightning"),
			"Iron Hard Skin defender must never receive the Lightning armor modifier");
		assertFalse(lightning.injuryContext().isArmorBroken(),
			"armor must stay unbroken since the needed bonus was denied");
	}
}
