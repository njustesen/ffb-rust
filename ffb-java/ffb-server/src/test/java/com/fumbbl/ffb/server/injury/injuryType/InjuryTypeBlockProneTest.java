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

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/injury/injuryType/injury_type_block_prone.rs tests.
 * A ModificationAware block that leaves a saved-by-armour player PRONE; a break rolls injury.
 * SendToBoxReason is BLOCKED. Unlike plain Block, injuryRoll() never adds a niggling injury
 * modifier (it only adds the value-0 Stunty marker). (The Rust initial_context_has_no_injury /
 * default_equivalent_to_new / new_context_uses_defender_apo_mode plumbing tests are Rust-structural
 * — exempt.)
 */
public class InjuryTypeBlockProneTest {

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
		GameFixture.placePlayer(gameState, "home1", 2, 2);
		GameFixture.placePlayer(gameState, "away1", 5, 5);
	}

	private RosterPlayer defender() {
		return (RosterPlayer) game.getPlayerById("away1");
	}

	private void handleInjury(InjuryTypeBlockProne blockProne) {
		blockProne.injuryContext().setDefenderId("away1");
		blockProne.handleInjury(step, game, gameState, gameState.getDiceRoller(), null, defender(),
			new FieldCoordinate(5, 5), null, null, ApothecaryMode.DEFENDER);
	}

	// rust: armor_save_results_in_prone
	@Test
	public void armorSaveResultsInProne() {
		GameFixture.installScriptedDice(gameState, 1, 1);
		InjuryTypeBlockProne blockProne = new InjuryTypeBlockProne();
		handleInjury(blockProne);
		assertFalse(blockProne.injuryContext().isArmorBroken());
		assertEquals(PlayerState.PRONE, blockProne.injuryContext().getInjury().getBase());
	}

	// rust: armor_break_results_in_injury_roll
	@Test
	public void armorBreakResultsInInjuryRoll() {
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBlockProne blockProne = new InjuryTypeBlockProne();
		handleInjury(blockProne);
		assertTrue(blockProne.injuryContext().isArmorBroken());
		assertNotEquals(PlayerState.PRONE, blockProne.injuryContext().getInjury().getBase());
	}

	// rust: send_to_box_reason_is_blocked
	@Test
	public void sendToBoxReasonIsBlocked() {
		assertEquals(SendToBoxReason.BLOCKED, new InjuryTypeBlockProne().sendToBoxReason());
	}

	// rust: niggling_injured_defender_gets_no_niggling_injury_modifier
	@Test
	public void nigglingInjuredDefenderGetsNoNigglingInjuryModifier() {
		init(GameFixture.createGameState(3, RulesCollection.Rules.BB2016));
		defender().addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE);
		GameFixture.installScriptedDice(gameState, 6, 6, 1, 1);
		InjuryTypeBlockProne blockProne = new InjuryTypeBlockProne();
		handleInjury(blockProne);
		assertFalse(Arrays.stream(blockProne.injuryContext().getInjuryModifiers())
			.anyMatch(m -> m.getName() != null && m.getName().contains("Niggling")));
	}
}
