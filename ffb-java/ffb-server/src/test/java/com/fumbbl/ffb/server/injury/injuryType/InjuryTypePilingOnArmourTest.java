package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.option.GameOptionBoolean;
import com.fumbbl.ffb.option.GameOptionId;
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
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_piling_on_armour.rs
 * tests. Piling On re-rolls the armour roll with no static bonus; the stacking branches are
 * driven by the PILING_ON_DOES_NOT_STACK game option.
 */
public class InjuryTypePilingOnArmourTest {

	private GameState gameState;
	private Game game;
	private IStep step;

	@BeforeEach
	void setUp() {
		init(GameFixture.createGameState(3));
	}

	private void init(GameState gs) {
		gameState = gs;
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

	private void enableOption(GameOptionId optionId) {
		GameOptionBoolean option =
			(GameOptionBoolean) game.getOptions().getFactory().createGameOption(optionId);
		option.setValue(true);
		game.getOptions().addOption(option);
	}

	private void handleInjury(InjuryTypePilingOnArmour pilingOn) {
		pilingOn.injuryContext().setAttackerId("home1");
		pilingOn.injuryContext().setDefenderId("away1");
		pilingOn.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasArmorModifier(InjuryTypePilingOnArmour pilingOn, String name) {
		return Arrays.stream(pilingOn.injuryContext().getArmorModifiers())
			.anyMatch(m -> name.equals(m.getName()));
	}

	private boolean hasInjuryModifier(InjuryTypePilingOnArmour pilingOn, String namePart) {
		return Arrays.stream(pilingOn.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	// rust: armor_save_results_in_prone
	@Test
	public void armorSaveResultsInProne() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypePilingOnArmour pilingOn = new InjuryTypePilingOnArmour();
		handleInjury(pilingOn);
		assertEquals(PlayerState.PRONE, pilingOn.injuryContext().getInjury().getBase());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypePilingOnArmour pilingOn = new InjuryTypePilingOnArmour();
		handleInjury(pilingOn);
		assertTrue(pilingOn.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, pilingOn.injuryContext().getInjury().getBase());
	}

	// rust: apo_allowed (bug #6: PilingOnArmour inherits canUseApo/fallingDownCausesTurnover = true)
	@Test
	public void apoAllowed() {
		InjuryTypePilingOnArmour pilingOn = new InjuryTypePilingOnArmour();
		assertTrue(pilingOn.canUseApo());
		assertTrue(pilingOn.injuryType().fallingDownCausesTurnover());
	}

	// rust: send_to_box_reason_is_piled_on
	@Test
	public void sendToBoxReasonIsPiledOn() {
		assertEquals(SendToBoxReason.PILED_ON, new InjuryTypePilingOnArmour().sendToBoxReason());
	}

	// rust: is_worth_spps_and_caused_by_opponent
	@Test
	public void isWorthSppsAndCausedByOpponent() {
		InjuryTypePilingOnArmour pilingOn = new InjuryTypePilingOnArmour();
		assertTrue(pilingOn.injuryType().isWorthSpps());
		assertTrue(pilingOn.injuryType().isCausedByOpponent());
	}

	// rust: no_static_piling_on_bonus_is_fabricated
	@Test
	public void noStaticPilingOnBonusIsFabricated() {
		defender().setArmour(7);
		GameFixture.installScriptedDice(gameState, 3, 3); // 6 < 7 -> not broken
		InjuryTypePilingOnArmour pilingOn = new InjuryTypePilingOnArmour();
		handleInjury(pilingOn);
		assertFalse(hasArmorModifier(pilingOn, "Piling On"));
		assertFalse(pilingOn.injuryContext().isArmorBroken(),
			"6 vs armour 7 with no skills must not break armour");
	}

	// rust: claw_armor_modifier_applied_when_not_broken_and_stacks_by_default
	@Test
	public void clawArmorModifierAppliedWhenNotBrokenAndStacksByDefault() {
		attacker().addSkill(GameFixture.skill(game, "Claws"));
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypePilingOnArmour pilingOn = new InjuryTypePilingOnArmour();
		handleInjury(pilingOn);
		assertTrue(hasArmorModifier(pilingOn, "Claws"),
			"Claws must be added from the attacker's own armor-modifier skills");
	}

	// rust: no_claw_skill_no_claws_modifier
	@Test
	public void noClawSkillNoClawsModifier() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypePilingOnArmour pilingOn = new InjuryTypePilingOnArmour();
		handleInjury(pilingOn);
		assertFalse(hasArmorModifier(pilingOn, "Claws"));
	}

	// rust: piling_on_does_not_stack_suppresses_all_armor_modifiers
	@Test
	public void pilingOnDoesNotStackSuppressesAllArmorModifiers() {
		enableOption(GameOptionId.PILING_ON_DOES_NOT_STACK);
		attacker().addSkill(GameFixture.skill(game, "Claws"));
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypePilingOnArmour pilingOn = new InjuryTypePilingOnArmour();
		handleInjury(pilingOn);
		assertEquals(0, pilingOn.injuryContext().getArmorModifiers().length,
			"PILING_ON_DOES_NOT_STACK must suppress all armor modifiers");
	}

	// rust: mighty_blow_adds_injury_modifier
	@Test
	public void mightyBlowAddsInjuryModifier() {
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypePilingOnArmour pilingOn = new InjuryTypePilingOnArmour();
		handleInjury(pilingOn);
		assertTrue(pilingOn.injuryContext().isArmorBroken());
		assertTrue(hasInjuryModifier(pilingOn, "Mighty Blow"));
	}

	// rust: no_mighty_blow_no_injury_modifier
	@Test
	public void noMightyBlowNoInjuryModifier() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypePilingOnArmour pilingOn = new InjuryTypePilingOnArmour();
		handleInjury(pilingOn);
		assertTrue(pilingOn.injuryContext().isArmorBroken());
		assertFalse(hasInjuryModifier(pilingOn, "Mighty Blow"));
	}

	// rust: piling_on_does_not_stack_suppresses_mighty_blow_but_not_niggling
	@Test
	public void pilingOnDoesNotStackSuppressesMightyBlowButNotNiggling() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		enableOption(GameOptionId.PILING_ON_DOES_NOT_STACK);
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		defender().setArmour(2);
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypePilingOnArmour pilingOn = new InjuryTypePilingOnArmour();
		handleInjury(pilingOn);
		assertTrue(pilingOn.injuryContext().isArmorBroken());
		assertFalse(hasInjuryModifier(pilingOn, "Mighty Blow"),
			"PILING_ON_DOES_NOT_STACK must suppress skill injury modifiers");
		assertTrue(hasInjuryModifier(pilingOn, "Niggling"),
			"niggling modifier is always added regardless of PILING_ON_DOES_NOT_STACK");
	}
}
