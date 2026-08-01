package com.fumbbl.ffb.server.util;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/util/util_server_re_roll.rs is*ReRollAvailable tests.
 * A skill-less player has no Pro re-roll; a team with no TRR (or in a re-roll-blocking turn mode)
 * has no team re-roll; an empty single-use pool has none.
 */
public class UtilServerReRollTest {

	private GameState gameState;
	private Game game;
	private Player<?> player;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		game.setHomePlaying(true);
		player = game.getPlayerById("home1");
	}

	// rust: pro_reroll_unavailable_without_skill
	@Test
	public void proReRollUnavailableWithoutSkill() {
		assertFalse(UtilServerReRoll.isProReRollAvailable(player, game, gameState.getPassState()));
	}

	// rust: team_reroll_unavailable_no_trr
	@Test
	public void teamReRollUnavailableNoTrr() {
		game.getTurnDataHome().setReRolls(0);
		assertFalse(UtilServerReRoll.isTeamReRollAvailable(gameState, player));
	}

	// rust: team_reroll_unavailable_on_kickoff_mode
	@Test
	public void teamReRollUnavailableOnKickoffMode() {
		GameFixture.setTurnMode(gameState, TurnMode.KICKOFF);
		game.getTurnDataHome().setReRolls(2);
		assertFalse(UtilServerReRoll.isTeamReRollAvailable(gameState, player));
	}

	// rust: bb2020_blocks_blitz_team_reroll (BB2020 blocks re-rolls during the Blitz kickoff event)
	@Test
	public void bb2020BlocksBlitzTeamReRoll() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		game.setHomePlaying(true);
		GameFixture.setTurnMode(gameState, TurnMode.BLITZ);
		game.getTurnDataHome().setReRolls(2);
		assertFalse(UtilServerReRoll.isTeamReRollAvailable(gameState, game.getPlayerById("home1")));
	}

	// rust: single_use_reroll_unavailable_when_empty
	@Test
	public void singleUseReRollUnavailableWhenEmpty() {
		assertFalse(UtilServerReRoll.isSingleUseReRollAvailable(gameState, player));
	}
}
