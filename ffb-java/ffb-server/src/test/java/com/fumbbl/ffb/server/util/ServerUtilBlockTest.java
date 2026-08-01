package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.PlayerState;
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

	/** As nrOfDice but with the canAddBlockDie modifier (7-arg overload returns a Pair). */
	private int nrOfDiceAddDie(int attStr, int defStr) {
		GameFixture.placePlayer(gameState, "away1", 13, 8);
		((RosterPlayer) game.getPlayerById("home1")).setStrength(attStr);
		((RosterPlayer) game.getPlayerById("away1")).setStrength(defStr);
		return ServerUtilBlock.findNrOfBlockDice(gameState, game.getPlayerById("home1"),
			game.getPlayerById("away1"), false, false, false, true).getLeft();
	}

	// rust: add_block_die_increments_one_die_to_two
	@Test
	public void addBlockDieIncrementsOneDieToTwo() {
		assertEquals(2, nrOfDiceAddDie(3, 3));
	}

	// rust: add_block_die_increments_two_dice_to_three
	@Test
	public void addBlockDieIncrementsTwoDiceToThree() {
		assertEquals(3, nrOfDiceAddDie(6, 3));
	}

	// rust: add_block_die_does_not_increment_three_dice
	@Test
	public void addBlockDieDoesNotIncrementThreeDice() {
		assertEquals(3, nrOfDiceAddDie(9, 3));
	}

	// ── removePlayerBlockStates ──────────────────────────────────────────────

	private int base(String playerId) {
		return game.getFieldModel().getPlayerState(game.getPlayerById(playerId)).getBase();
	}

	private void setBase(String playerId, int newBase) {
		PlayerState ps = game.getFieldModel().getPlayerState(game.getPlayerById(playerId));
		game.getFieldModel().setPlayerState(game.getPlayerById(playerId), ps.changeBase(newBase));
	}

	// rust: remove_player_block_states_resets_blocked_to_standing
	@Test
	public void removePlayerBlockStatesResetsBlockedToStanding() {
		GameFixture.placePlayer(gameState, "home2", 4, 4);
		setBase("home1", PlayerState.BLOCKED);
		setBase("home2", PlayerState.STANDING);
		ServerUtilBlock.removePlayerBlockStates(game, null);
		assertEquals(PlayerState.STANDING, base("home1"));
		assertEquals(PlayerState.STANDING, base("home2"));
	}

	// rust: remove_player_block_states_preserves_prone_defender
	@Test
	public void removePlayerBlockStatesPreservesProneDefender() {
		GameFixture.placePlayer(gameState, "away1", 13, 8);
		game.setDefenderId("away1");
		setBase("away1", PlayerState.BLOCKED);
		ServerUtilBlock.removePlayerBlockStates(game, new PlayerState(PlayerState.PRONE));
		assertEquals(PlayerState.PRONE, base("away1"));
	}

	// rust: remove_player_block_states_non_defender_always_standing
	@Test
	public void removePlayerBlockStatesNonDefenderAlwaysStanding() {
		GameFixture.placePlayer(gameState, "away1", 13, 8);
		game.setDefenderId("away1");
		setBase("home1", PlayerState.BLOCKED);
		setBase("away1", PlayerState.BLOCKED);
		ServerUtilBlock.removePlayerBlockStates(game, new PlayerState(PlayerState.PRONE));
		assertEquals(PlayerState.STANDING, base("home1"));
	}

	// rust: remove_player_block_states_non_blocked_untouched
	@Test
	public void removePlayerBlockStatesNonBlockedUntouched() {
		setBase("home1", PlayerState.PRONE);
		ServerUtilBlock.removePlayerBlockStates(game, null);
		assertEquals(PlayerState.PRONE, base("home1"));
	}
}
