package com.fumbbl.ffb.server.inducements.mixed.prayers;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.AnimationType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.RosterPlayer;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust
 * crates/ffb-engine/src/inducements/mixed/prayers/intensive_training_handler.rs tests (portable
 * subset — the Rust headless init_effect random-selection test has no Java twin; Java shows a
 * skill-choice dialog instead).
 */
public class IntensiveTrainingHandlerTest {

	private GameState gameState;
	private Game game;
	private IntensiveTrainingHandler handler;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		handler = new com.fumbbl.ffb.server.inducements.bb2020.prayers.IntensiveTrainingHandler();
	}

	private boolean hasTempSkill(RosterPlayer player, String skillName) {
		return player.getSkillsIncludingTemporaryOnes().stream()
			.anyMatch(s -> skillName.equals(s.getName()));
	}

	// rust: animation_type_is_correct
	@Test
	public void animationTypeIsCorrect() {
		assertEquals(AnimationType.PRAYER_INTENSIVE_TRAINING, handler.animationType());
	}

	// rust: init_effect_returns_true (stub selector = no eligible players -> prayer wasted)
	@Test
	public void initEffectReturnsTrueWhenNoEligiblePlayers() {
		gameState = GameFixture.createGameState(0, RulesCollection.Rules.BB2020);
		game = gameState.getGame();
		assertTrue(handler.initEffect(gameState, game.getTeamHome()));
	}

	// rust: remove_effect_clears_enhancement (Java: the dialog-applied intensive-training skill
	// is stored under the prayer's name and removeEffectInternal clears it via the selector)
	@Test
	public void removeEffectClearsEnhancement() {
		RosterPlayer player = (RosterPlayer) game.getPlayerById("home1");
		Skill dodge = GameFixture.skill(game, "Dodge");
		game.getFieldModel().addIntensiveTrainingSkill("home1", dodge);
		assertTrue(hasTempSkill(player, "Dodge"));
		handler.removeEffectInternal(gameState, game.getTeamHome());
		assertFalse(hasTempSkill(player, "Dodge"));
	}
}
