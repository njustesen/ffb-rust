package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/util/server_util_block.rs block-dice-count tests. The
 * Rust helper takes bare strengths; here we build two players with those strengths (isolated, no
 * assisting team-mates on the field) and call ServerUtilBlock.findNrOfBlockDice.
 */
public class ServerUtilBlockTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 13, 7);
	}

	/** attacker home1 with attStr; defender at (13,8); same-team uses home2, else away1. */
	private int nrOfDice(int attStr, int defStr, boolean sameTeam, boolean multiBlock) {
		String defenderId = sameTeam ? "home2" : "away1";
		GameFixture.placePlayer(gameState, defenderId, 13, 8);
		((RosterPlayer) game.getPlayerById("home1")).setStrength(attStr);
		((RosterPlayer) game.getPlayerById(defenderId)).setStrength(defStr);
		Player<?> attacker = game.getPlayerById("home1");
		Player<?> defender = game.getPlayerById(defenderId);
		return ServerUtilBlock.findNrOfBlockDice(gameState, attacker, defender, multiBlock, false);
	}

	// rust: equal_strength_gives_one_die
	@Test
	public void equalStrengthGivesOneDie() {
		assertEquals(1, nrOfDice(3, 3, false, false));
	}

	// rust: double_attacker_strength_gives_two_dice
	@Test
	public void doubleAttackerStrengthGivesTwoDice() {
		assertEquals(2, nrOfDice(6, 3, false, false));
	}

	// rust: triple_attacker_strength_gives_three_dice
	@Test
	public void tripleAttackerStrengthGivesThreeDice() {
		assertEquals(3, nrOfDice(7, 3, false, false));
	}

	// rust: weaker_attacker_gives_minus_two
	@Test
	public void weakerAttackerGivesMinusTwo() {
		assertEquals(-2, nrOfDice(3, 4, false, false));
	}

	// rust: much_weaker_attacker_gives_minus_three
	@Test
	public void muchWeakerAttackerGivesMinusThree() {
		assertEquals(-3, nrOfDice(3, 7, false, false));
	}

	// rust: same_team_block_always_positive (Ball & Chain vs own team: -2 becomes +2)
	@Test
	public void sameTeamBlockAlwaysPositive() {
		assertEquals(2, nrOfDice(3, 4, true, false));
	}

	// rust: multi_block_applies_edition_modifiers (bb2025 applies the ATTACKER -2 penalty)
	@Test
	public void multiBlockAppliesAttackerPenaltyBb2025() {
		// attacker 5 - 2 = 3 == defender 3 -> 1 die
		assertEquals(1, nrOfDice(5, 3, false, true));
		// attacker 6 - 2 = 4 > defender 3 -> 2 dice
		assertEquals(2, nrOfDice(6, 3, false, true));
	}
}
