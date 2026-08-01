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
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_piling_on_injury.rs
 * tests. The injury re-roll variant: armour is treated as already broken.
 */
public class InjuryTypePilingOnInjuryTest {

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

	private void handleInjury(InjuryTypePilingOnInjury pilingOn) {
		pilingOn.injuryContext().setAttackerId("home1");
		pilingOn.injuryContext().setDefenderId("away1");
		pilingOn.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasInjuryModifier(InjuryTypePilingOnInjury pilingOn, String namePart) {
		return Arrays.stream(pilingOn.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	// rust: armor_already_broken_and_injury_rolled
	@Test
	public void armorAlreadyBrokenAndInjuryRolled() {
		GameFixture.installScriptedDice(gameState, 1, 1); // injury 2 -> stunned
		InjuryTypePilingOnInjury pilingOn = new InjuryTypePilingOnInjury();
		handleInjury(pilingOn);
		assertTrue(pilingOn.injuryContext().isArmorBroken());
		assertNotNull(pilingOn.injuryContext().getInjury());
		assertNotEquals(PlayerState.PRONE, pilingOn.injuryContext().getInjury().getBase());
	}

	// rust: apo_allowed (bug #6: PilingOnInjury inherits canUseApo = true)
	@Test
	public void apoAllowed() {
		assertTrue(new InjuryTypePilingOnInjury().canUseApo());
	}

	// rust: send_to_box_reason_is_piled_on
	@Test
	public void sendToBoxReasonIsPiledOn() {
		assertEquals(SendToBoxReason.PILED_ON, new InjuryTypePilingOnInjury().sendToBoxReason());
	}

	// rust: is_worth_spps_and_caused_by_opponent
	@Test
	public void isWorthSppsAndCausedByOpponent() {
		InjuryTypePilingOnInjury pilingOn = new InjuryTypePilingOnInjury();
		assertTrue(pilingOn.injuryType().isWorthSpps());
		assertTrue(pilingOn.injuryType().isCausedByOpponent());
	}

	// rust: turnover_default_true
	@Test
	public void turnoverDefaultTrue() {
		assertTrue(new InjuryTypePilingOnInjury().injuryType().fallingDownCausesTurnover());
	}

	// rust: niggling_injured_defender_gets_niggling_injury_modifier
	@Test
	public void nigglingInjuredDefenderGetsNigglingInjuryModifier() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypePilingOnInjury pilingOn = new InjuryTypePilingOnInjury();
		handleInjury(pilingOn);
		assertTrue(hasInjuryModifier(pilingOn, "Niggling"));
	}

	// rust: non_niggling_defender_gets_no_niggling_injury_modifier
	@Test
	public void nonNigglingDefenderGetsNoNigglingInjuryModifier() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypePilingOnInjury pilingOn = new InjuryTypePilingOnInjury();
		handleInjury(pilingOn);
		assertFalse(hasInjuryModifier(pilingOn, "Niggling"));
	}

	// rust: mighty_blow_adds_injury_modifier_during_piling_on_injury_reroll
	@Test
	public void mightyBlowAddsInjuryModifierDuringPilingOnInjuryReroll() {
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypePilingOnInjury pilingOn = new InjuryTypePilingOnInjury();
		handleInjury(pilingOn);
		assertTrue(hasInjuryModifier(pilingOn, "Mighty Blow"));
	}

	// rust: no_mighty_blow_no_injury_modifier
	@Test
	public void noMightyBlowNoInjuryModifier() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypePilingOnInjury pilingOn = new InjuryTypePilingOnInjury();
		handleInjury(pilingOn);
		assertFalse(hasInjuryModifier(pilingOn, "Mighty Blow"));
	}

	// rust: piling_on_does_not_stack_suppresses_mighty_blow_but_not_niggling
	@Test
	public void pilingOnDoesNotStackSuppressesMightyBlowButNotNiggling() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		enableOption(GameOptionId.PILING_ON_DOES_NOT_STACK);
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypePilingOnInjury pilingOn = new InjuryTypePilingOnInjury();
		handleInjury(pilingOn);
		assertFalse(hasInjuryModifier(pilingOn, "Mighty Blow"));
		assertTrue(hasInjuryModifier(pilingOn, "Niggling"));
	}
}
