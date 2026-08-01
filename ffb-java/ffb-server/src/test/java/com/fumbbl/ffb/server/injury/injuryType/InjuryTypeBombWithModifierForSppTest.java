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
 * Mirror of ffb-rust
 * crates/ffb-engine/src/injury/injuryType/injury_type_bomb_with_modifier_for_spp.rs tests.
 * BombForSpp overrides isCausedByOpponent to true, so AbstractInjuryTypeBombWithModifier passes
 * the REAL attacker to findInjuryModifiers — attacker-sourced injury modifiers like Mighty Blow
 * DO apply (the inverse of the base Bomb variant).
 */
public class InjuryTypeBombWithModifierForSppTest {

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
		// the armour-modifier factory caches the bombUsesMb flag in initialize(game), which ran
		// before the option was set - re-initialize so setUseAll picks it up
		game.getFactory(com.fumbbl.ffb.FactoryType.Factory.ARMOUR_MODIFIER).initialize(game);
	}

	private void handleInjury(InjuryTypeBombWithModifierForSpp bomb) {
		bomb.injuryContext().setAttackerId("home1");
		bomb.injuryContext().setDefenderId("away1");
		bomb.handleInjury(step, game, gameState, gameState.getDiceRoller(), attacker(), defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasArmorModifier(InjuryTypeBombWithModifierForSpp bomb, String namePart) {
		return Arrays.stream(bomb.injuryContext().getArmorModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	private boolean hasInjuryModifier(InjuryTypeBombWithModifierForSpp bomb, String namePart) {
		return Arrays.stream(bomb.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains(namePart));
	}

	// rust: armor_save_results_in_prone
	@Test
	public void armorSaveResultsInProne() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeBombWithModifierForSpp bomb = new InjuryTypeBombWithModifierForSpp();
		handleInjury(bomb);
		assertFalse(bomb.injuryContext().isArmorBroken());
		assertNotNull(bomb.injuryContext().getInjury());
		assertEquals(PlayerState.PRONE, bomb.injuryContext().getInjury().getBase());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBombWithModifierForSpp bomb = new InjuryTypeBombWithModifierForSpp();
		handleInjury(bomb);
		assertTrue(bomb.injuryContext().isArmorBroken());
		assertNotNull(bomb.injuryContext().getInjury());
		assertNotEquals(PlayerState.PRONE, bomb.injuryContext().getInjury().getBase());
	}

	// rust: turnover_default_true
	@Test
	public void turnoverDefaultTrue() {
		assertTrue(new InjuryTypeBombWithModifierForSpp().injuryType().fallingDownCausesTurnover());
	}

	// rust: is_worth_spps_and_caused_by_opponent
	@Test
	public void isWorthSppsAndCausedByOpponent() {
		InjuryTypeBombWithModifierForSpp bomb = new InjuryTypeBombWithModifierForSpp();
		assertTrue(bomb.injuryType().isWorthSpps());
		assertTrue(bomb.injuryType().isCausedByOpponent());
	}

	// rust: send_to_box_reason_is_bomb
	@Test
	public void sendToBoxReasonIsBomb() {
		assertEquals(SendToBoxReason.BOMB, new InjuryTypeBombWithModifierForSpp().sendToBoxReason());
	}

	// rust: no_bomb_armor_modifier_under_bb2025_default_options
	@Test
	public void noBombArmorModifierUnderBb2025DefaultOptions() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeBombWithModifierForSpp bomb = new InjuryTypeBombWithModifierForSpp();
		handleInjury(bomb);
		assertEquals(0, bomb.injuryContext().getArmorModifiers().length);
	}

	// rust: bomb_armor_modifier_is_added_under_bb2020_when_bomb_uses_mb_enabled
	@Test
	public void bombArmorModifierIsAddedUnderBb2020WhenBombUsesMbEnabled() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2020));
		enableOption(GameOptionId.BOMB_USES_MB);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeBombWithModifierForSpp bomb = new InjuryTypeBombWithModifierForSpp();
		handleInjury(bomb);
		assertTrue(hasArmorModifier(bomb, "Bomb"));
	}

	// rust: attacker_mighty_blow_applies_because_bomb_for_spp_is_caused_by_opponent (bug #10:
	// BombForSpp overrides isCausedByOpponent to true, so handleInjury passes the real attacker)
	@Test
	public void attackerMightyBlowAppliesBecauseBombForSppIsCausedByOpponent() {
		attacker().addSkill(GameFixture.skill(game, "Mighty Blow"));
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBombWithModifierForSpp bomb = new InjuryTypeBombWithModifierForSpp();
		handleInjury(bomb);
		assertTrue(bomb.injuryContext().isArmorBroken());
		assertTrue(hasInjuryModifier(bomb, "Mighty Blow"));
	}

	// rust: niggling_injury_modifier_still_applies
	@Test
	public void nigglingInjuryModifierStillApplies() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBombWithModifierForSpp bomb = new InjuryTypeBombWithModifierForSpp();
		handleInjury(bomb);
		assertTrue(bomb.injuryContext().isArmorBroken());
		assertTrue(hasInjuryModifier(bomb, "Niggling"));
	}
}
