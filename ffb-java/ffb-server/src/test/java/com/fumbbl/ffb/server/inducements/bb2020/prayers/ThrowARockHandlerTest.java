package com.fumbbl.ffb.server.inducements.bb2020.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.bb2020.Prayer;
import com.fumbbl.ffb.model.AnimationType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/inducements/bb2020/prayers/throw_a_rock_handler.rs
 * tests. THROW_A_ROCK (bb2020) marks the OPPONENT team as should-not-stall.
 */
public class ThrowARockHandlerTest {

	private GameState gameState;
	private Game game;
	private ThrowARockHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new ThrowARockHandler();
	}

	// rust: handles_prayer_throw_a_rock
	@Test
	public void handlesPrayerThrowARock() {
		assertEquals(Prayer.THROW_A_ROCK, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.THROW_A_ROCK));
		assertFalse(handler.handles(Prayer.IRON_MAN));
	}

	// rust: init_effect_adds_should_not_stall_to_other_team
	@Test
	public void initEffectAddsShouldNotStallToOtherTeam() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
		assertTrue(gameState.getPrayerState().shouldNotStall(game.getTeamAway()));
		assertFalse(gameState.getPrayerState().shouldNotStall(game.getTeamHome()));
	}

	// rust: init_effect_away_team_praying_marks_home
	@Test
	public void initEffectAwayTeamPrayingMarksHome() {
		assertTrue(handler.initEffect(gameState, game.getTeamAway()));
		assertTrue(gameState.getPrayerState().shouldNotStall(game.getTeamHome()));
		assertFalse(gameState.getPrayerState().shouldNotStall(game.getTeamAway()));
	}

	// rust: remove_effect_clears_should_not_stall
	@Test
	public void removeEffectClearsShouldNotStall() {
		gameState.getPrayerState().addShouldNotStall(game.getTeamAway());
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(gameState.getPrayerState().shouldNotStall(game.getTeamAway()));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_THROW_A_ROCK, handler.animationType());
	}
}
