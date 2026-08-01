package com.fumbbl.ffb.server.skillbehaviour.bb2025;

import com.fumbbl.ffb.ReRolledActions;
import com.fumbbl.ffb.ReRollSources;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.net.commands.ClientCommandUseSkill;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.model.StepModifier;
import com.fumbbl.ffb.server.step.StepCommandStatus;
import com.fumbbl.ffb.server.step.bb2025.ttm.StepThrowTeamMate;
import com.fumbbl.ffb.server.step.bb2025.pass.StepHailMaryPass;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/skill_behaviour/bb2025/the_ballista_behaviour.rs tests
 * (portable subset — registry/applies_to/execute-returns-false plumbing exempt). The Ballista
 * grants a reroll for TTM/KTM and Hail Mary Pass via the command hook, cleared when declined.
 */
public class TheBallistaBehaviourTest {

	private GameState gameState;
	private Game game;
	private TheBallistaBehaviour behaviour;

	@BeforeEach
	void setUp() {
		gameState = GameFixture.createGameState(3, RulesCollection.Rules.BB2025);
		game = gameState.getGame();
		behaviour = new TheBallistaBehaviour();
		behaviour.skill = (com.fumbbl.ffb.skill.bb2025.special.TheBallista) GameFixture.skill(game, "The Ballista");
	}

	private ClientCommandUseSkill useSkill(boolean skillUsed) {
		return new ClientCommandUseSkill(GameFixture.skill(game, "The Ballista"), skillUsed, "home1", null, false);
	}

	@SuppressWarnings({"unchecked", "rawtypes"})
	private StepModifier ttmModifier() {
		for (StepModifier m : behaviour.getStepModifiers()) {
			if (m.getConcreteClass() == StepThrowTeamMate.class) {
				return m;
			}
		}
		throw new IllegalStateException("no TTM modifier");
	}

	@SuppressWarnings({"unchecked", "rawtypes"})
	private StepModifier hmpModifier() {
		for (StepModifier m : behaviour.getStepModifiers()) {
			if (m.getConcreteClass() == StepHailMaryPass.class) {
				return m;
			}
		}
		throw new IllegalStateException("no HMP modifier");
	}

	// rust: throw_team_mate_handle_command_sets_throw_team_mate_when_not_kicked
	@Test
	@SuppressWarnings({"unchecked", "rawtypes"})
	public void ttmCommandSetsThrowTeamMateWhenNotKicked() {
		StepThrowTeamMate step = new StepThrowTeamMate(gameState);
		StepThrowTeamMate.StepState state = new StepThrowTeamMate.StepState();
		state.kicked = false;
		StepCommandStatus status = ttmModifier().handleCommandHook(step, state, useSkill(true));
		assertEquals(StepCommandStatus.EXECUTE_STEP, status);
		assertEquals(ReRolledActions.THROW_TEAM_MATE, step.getReRolledAction());
		assertEquals(ReRollSources.THE_BALLISTA, step.getReRollSource());
	}

	// rust: throw_team_mate_handle_command_sets_kick_team_mate_when_kicked
	@Test
	@SuppressWarnings({"unchecked", "rawtypes"})
	public void ttmCommandSetsKickTeamMateWhenKicked() {
		StepThrowTeamMate step = new StepThrowTeamMate(gameState);
		StepThrowTeamMate.StepState state = new StepThrowTeamMate.StepState();
		state.kicked = true;
		ttmModifier().handleCommandHook(step, state, useSkill(true));
		assertEquals(ReRolledActions.KICK_TEAM_MATE, step.getReRolledAction());
	}

	// rust: throw_team_mate_handle_command_clears_source_when_declined
	@Test
	@SuppressWarnings({"unchecked", "rawtypes"})
	public void ttmCommandClearsSourceWhenDeclined() {
		StepThrowTeamMate step = new StepThrowTeamMate(gameState);
		StepThrowTeamMate.StepState state = new StepThrowTeamMate.StepState();
		ttmModifier().handleCommandHook(step, state, useSkill(false));
		assertEquals(ReRolledActions.THROW_TEAM_MATE, step.getReRolledAction());
		assertNull(step.getReRollSource());
	}

	// rust: hail_mary_pass_handle_command_sets_pass_action
	@Test
	@SuppressWarnings({"unchecked", "rawtypes"})
	public void hmpCommandSetsPassAction() {
		StepHailMaryPass step = new StepHailMaryPass(gameState);
		StepHailMaryPass.StepState state = new StepHailMaryPass.StepState();
		StepCommandStatus status = hmpModifier().handleCommandHook(step, state, useSkill(true));
		assertEquals(StepCommandStatus.EXECUTE_STEP, status);
		assertEquals(ReRolledActions.PASS, step.getReRolledAction());
		assertEquals(ReRollSources.THE_BALLISTA, step.getReRollSource());
	}

	// rust: hail_mary_pass_handle_command_clears_source_when_declined
	@Test
	@SuppressWarnings({"unchecked", "rawtypes"})
	public void hmpCommandClearsSourceWhenDeclined() {
		StepHailMaryPass step = new StepHailMaryPass(gameState);
		StepHailMaryPass.StepState state = new StepHailMaryPass.StepState();
		hmpModifier().handleCommandHook(step, state, useSkill(false));
		assertEquals(ReRolledActions.PASS, step.getReRolledAction());
		assertNull(step.getReRollSource());
	}
}
