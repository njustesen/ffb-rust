package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
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
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_projectile_vomit.rs tests.
 */
public class InjuryTypeProjectileVomitTest {

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
		GameFixture.placePlayer(gameState, "away1", 5, 5); // defender
	}

	private RosterPlayer defender() {
		return (RosterPlayer) game.getPlayerById("away1");
	}

	private void handleInjury(InjuryTypeProjectileVomit injuryType) {
		injuryType.injuryContext().setDefenderId("away1");
		injuryType.handleInjury(step, game, gameState, gameState.getDiceRoller(), null, defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	private boolean hasNigglingModifier(InjuryTypeProjectileVomit injuryType) {
		return Arrays.stream(injuryType.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains("Niggling"));
	}

	// rust: armor_save_leaves_no_injury
	@Test
	public void armorSaveLeavesNoInjury() {
		defender().setArmour(13);
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeProjectileVomit injuryType = new InjuryTypeProjectileVomit();
		handleInjury(injuryType);
		assertFalse(injuryType.injuryContext().isArmorBroken());
		assertNull(injuryType.injuryContext().getInjury());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeProjectileVomit injuryType = new InjuryTypeProjectileVomit();
		handleInjury(injuryType);
		assertTrue(injuryType.injuryContext().isArmorBroken());
		assertNotNull(injuryType.injuryContext().getInjury());
	}

	// rust: turnover_default_true (flag audit: no Java override, base default true)
	@Test
	public void turnoverDefaultTrue() {
		assertTrue(new InjuryTypeProjectileVomit().injuryType().fallingDownCausesTurnover());
	}

	// rust: send_to_box_reason_is_projectile_vomit
	@Test
	public void sendToBoxReason() {
		assertEquals(SendToBoxReason.PROJECTILE_VOMIT, new InjuryTypeProjectileVomit().sendToBoxReason());
	}

	// rust: is_caused_by_opponent
	@Test
	public void isCausedByOpponent() {
		assertTrue(new InjuryTypeProjectileVomit().injuryType().isCausedByOpponent());
	}

	// rust: failed_armour_does_not_place_prone
	@Test
	public void failedArmourDoesNotPlaceProne() {
		assertFalse(new InjuryTypeProjectileVomit().injuryType().failedArmourPlacesProne());
	}

	// rust: niggling_injury_modifier_applied_when_armor_breaks
	@Test
	public void nigglingInjuryModifierAppliedWhenArmorBreaks() {
		init(GameFixture.createGameState(3, com.fumbbl.ffb.RulesCollection.Rules.BB2016));
		defender().setArmour(2);
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeProjectileVomit injuryType = new InjuryTypeProjectileVomit();
		handleInjury(injuryType);
		assertTrue(injuryType.injuryContext().isArmorBroken());
		assertTrue(hasNigglingModifier(injuryType));
	}

	// rust: no_niggling_injury_no_modifier
	@Test
	public void noNigglingInjuryNoModifier() {
		init(GameFixture.createGameState(3, com.fumbbl.ffb.RulesCollection.Rules.BB2016));
		defender().setArmour(2);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeProjectileVomit injuryType = new InjuryTypeProjectileVomit();
		handleInjury(injuryType);
		assertTrue(injuryType.injuryContext().isArmorBroken());
		assertFalse(hasNigglingModifier(injuryType));
	}
}
