package com.fumbbl.ffb.server.injury.injuryType;

import com.fumbbl.ffb.ApothecaryMode;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_sabotaged.rs tests.
 * A sabotaged trap injury: standard armour + injury roll but NO block/skill modifiers (Sabotaged
 * skips findInjuryModifiers). Armour save leaves the player prone; a break rolls injury.
 * SendToBoxReason SABOTAGED, not caused by opponent. (The Rust default_equivalent_to_new plumbing
 * test is Rust-structural — exempt.)
 */
public class InjuryTypeSabotagedTest {

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
		GameFixture.placePlayer(gameState, "away1", 5, 5);
	}

	private RosterPlayer defender() {
		return (RosterPlayer) game.getPlayerById("away1");
	}

	private void handleInjury(InjuryTypeSabotaged sabotaged) {
		sabotaged.injuryContext().setDefenderId("away1");
		sabotaged.handleInjury(step, game, gameState, gameState.getDiceRoller(), null, defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	// rust: armor_save_results_in_prone
	@Test
	public void armorSaveResultsInProne() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeSabotaged sabotaged = new InjuryTypeSabotaged();
		handleInjury(sabotaged);
		assertEquals(PlayerState.PRONE, sabotaged.injuryContext().getInjury().getBase());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeSabotaged sabotaged = new InjuryTypeSabotaged();
		handleInjury(sabotaged);
		assertTrue(sabotaged.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, sabotaged.injuryContext().getInjury().getBase());
	}

	// rust: causes_turnover_by_default
	@Test
	public void causesTurnoverByDefault() {
		assertTrue(new InjuryTypeSabotaged().injuryType().fallingDownCausesTurnover());
	}

	// rust: send_to_box_reason_is_sabotaged
	@Test
	public void sendToBoxReasonIsSabotaged() {
		assertEquals(SendToBoxReason.SABOTAGED, new InjuryTypeSabotaged().sendToBoxReason());
	}

	// Sabotaged does not cause SPPs / is not caused by an opponent (trap, not a player action)
	@Test
	public void notCausedByOpponent() {
		assertEquals(false, new InjuryTypeSabotaged().injuryType().isCausedByOpponent());
	}

	// rust: context_stores_attacker_and_defender (Java handleInjury does not populate the context
	// ids — the CALLER sets them; assert the defender id the test set)
	@Test
	public void contextStoresDefender() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeSabotaged sabotaged = new InjuryTypeSabotaged();
		handleInjury(sabotaged);
		assertEquals("away1", sabotaged.injuryContext().getDefenderId());
	}

	// rust: stunty_defender_stunned_at_total_7_bb2020_no_modifiers — Sabotaged skips
	// findInjuryModifiers, so the Stunty injury modifier is NOT in the context and
	// interpretInjuryRoll's isStunty is false: a total of 7 stays STUNNED (not KO).
	@Test
	public void stuntyDefenderStunnedAtTotal7Bb2020() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2020));
		defender().addSkill(GameFixture.skill(game, "Stunty"));
		GameFixture.installScriptedDice(gameState, 6, 6, 3, 4); // armour break, injury total 7
		InjuryTypeSabotaged sabotaged = new InjuryTypeSabotaged();
		handleInjury(sabotaged);
		assertEquals(PlayerState.STUNNED, sabotaged.injuryContext().getInjury().getBase());
	}
}
