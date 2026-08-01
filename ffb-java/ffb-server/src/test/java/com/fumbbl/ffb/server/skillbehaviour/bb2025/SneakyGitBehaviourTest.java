package com.fumbbl.ffb.server.skillbehaviour.bb2025;

import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SendToBoxReason;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.PlayerResult;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.model.StepModifier;
import com.fumbbl.ffb.server.step.mixed.foul.StepEjectPlayer;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2025/sneaky_git_behaviour.rs tests
 * (eject-player subset; referee-modifier and registry plumbing exempt). A fouler who is ejected
 * is set BANNED (unless Argue-the-Call succeeded) and the reason is recorded on their
 * PlayerResult (OFFICIOUS_REF / THREW_TWO_BOMBS / FOUL_BAN).
 */
public class SneakyGitBehaviourTest {

	private GameState gameState;
	private Game game;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		GameFixture.placePlayer(gameState, "home1", 5, 5);
		GameFixture.setActingPlayer(gameState, "home1", PlayerAction.FOUL);
	}

	private StepEjectPlayer.StepState newState(Boolean argueSuccessful, boolean officiousRef) {
		StepEjectPlayer.StepState state = new StepEjectPlayer.StepState();
		state.argueTheCallSuccessful = argueSuccessful;
		state.officiousRef = officiousRef;
		return state;
	}

	@SuppressWarnings({"unchecked", "rawtypes"})
	private boolean executeHook(StepEjectPlayer.StepState state) {
		StepEjectPlayer step = new StepEjectPlayer(gameState);
		SneakyGitBehaviour behaviour = new SneakyGitBehaviour();
		behaviour.skill = (com.fumbbl.ffb.skill.bb2025.SneakyGit) GameFixture.skill(game, "Sneaky Git");
		StepModifier modifier = null;
		for (StepModifier m : behaviour.getStepModifiers()) {
			if (m.getConcreteClass() == StepEjectPlayer.class) {
				modifier = m;
				break;
			}
		}
		return modifier.handleExecuteStepHook(step, state);
	}

	private int base() {
		return game.getFieldModel().getPlayerState(game.getPlayerById("home1")).getBase();
	}

	private PlayerResult foulerResult() {
		return game.getGameResult().getPlayerResult(game.getPlayerById("home1"));
	}

	// rust: no_argue_the_call_and_not_cased_sets_banned
	@Test
	public void noArgueAndNotCasedSetsBanned() {
		executeHook(newState(null, false));
		assertEquals(PlayerState.BANNED, base());
	}

	// rust: argue_the_call_successful_prevents_ban
	@Test
	public void argueTheCallSuccessfulPreventsBan() {
		executeHook(newState(true, false));
		assertNotEquals(PlayerState.BANNED, base());
	}

	// rust: foul_ban_reason_is_default
	@Test
	public void foulBanReasonIsDefault() {
		executeHook(newState(null, false));
		assertEquals(SendToBoxReason.FOUL_BAN, foulerResult().getSendToBoxReason());
	}

	// rust: officious_ref_reason_is_set_correctly
	@Test
	public void officiousRefReasonIsSetCorrectly() {
		executeHook(newState(null, true));
		assertEquals(SendToBoxReason.OFFICIOUS_REF, foulerResult().getSendToBoxReason());
	}

	// rust: threw_two_bombs_reason_when_original_bombardier
	@Test
	public void threwTwoBombsReasonWhenOriginalBombardier() {
		gameState.getPassState().setOriginalBombardier("home1");
		executeHook(newState(null, false));
		assertEquals(SendToBoxReason.THREW_TWO_BOMBS, foulerResult().getSendToBoxReason());
	}

	// rust: send_to_box_turn_and_half_recorded
	@Test
	public void sendToBoxTurnAndHalfRecorded() {
		executeHook(newState(null, false));
		assertEquals(game.getTurnData().getTurnNr(), foulerResult().getSendToBoxTurn());
		assertEquals(game.getHalf(), foulerResult().getSendToBoxHalf());
	}

	// rust: modifier_returns_false
	@Test
	public void modifierReturnsFalse() {
		assertFalse(executeHook(newState(null, false)));
	}
}
