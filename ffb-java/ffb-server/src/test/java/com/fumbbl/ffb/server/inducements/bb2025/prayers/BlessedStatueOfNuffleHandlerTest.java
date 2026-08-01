package com.fumbbl.ffb.server.inducements.bb2025.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.inducement.bb2025.Prayer;
import com.fumbbl.ffb.model.AnimationType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust
 * crates/ffb-engine/src/inducements/bb2025/prayers/blessed_statue_of_nuffle_handler.rs tests.
 * The BB2025 Java handler extends RandomSelectionPrayerHandler directly and handles the
 * DIFFERENT prayer id BLESSING_OF_NUFFLE (bug #11 in the Rust port) — it random-selects and
 * applies immediately, so the grant IS portable here (unlike the bb2020 dialog variant).
 */
public class BlessedStatueOfNuffleHandlerTest {

	private GameState gameState;
	private Game game;
	private BlessedStatueOfNuffleHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		handler = new BlessedStatueOfNuffleHandler();
	}

	private boolean hasPro(Player<?> player) {
		return player.getSkillsIncludingTemporaryOnes().stream()
			.anyMatch(s -> "Pro".equals(s.getName()));
	}

	// rust: handles_prayer_blessing_of_nuffle (bug #11: bb2025 id is BLESSING_OF_NUFFLE)
	@Test
	public void handlesPrayerBlessingOfNuffle() {
		assertEquals(Prayer.BLESSING_OF_NUFFLE, handler.handledPrayer());
		assertTrue(handler.handles(Prayer.BLESSING_OF_NUFFLE));
		assertFalse(handler.handles(Prayer.IRON_MAN));
	}

	// rust: init_effect_returns_true
	@Test
	public void initEffectReturnsTrue() {
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: init_effect_grants_pro_to_reserve_player
	@Test
	public void initEffectGrantsProToSelectedPlayer() {
		handler.initEffect(gameState, game.getTeamHome());
		boolean anyHomeEnhanced = Arrays.stream(game.getTeamHome().getPlayers())
			.anyMatch(p -> p.hasActiveEnhancement(Prayer.BLESSING_OF_NUFFLE.getName()) && hasPro(p));
		assertTrue(anyHomeEnhanced);
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_BLESSED_STATUE_OF_NUFFLE, handler.animationType());
	}

	// rust: does_not_handle_other_prayers
	@Test
	public void doesNotHandleOtherPrayers() {
		assertFalse(handler.handles(Prayer.PERFECT_PASSING));
	}
}
