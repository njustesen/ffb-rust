package com.fumbbl.ffb.server.factory;

import com.fumbbl.ffb.server.step.StepId;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/step/framework/step_id_factory.rs tests. forName does a
 * case-insensitive lookup by StepId.getName() first, then by StepId.getOldName(); unknown names
 * return null.
 */
public class StepIdFactoryTest {

	private final StepIdFactory factory = new StepIdFactory();

	// rust: for_name_init_start_game
	@Test
	public void forNameInitStartGame() {
		assertEquals(StepId.INIT_START_GAME, factory.forName("initStartGame"));
	}

	// rust: for_name_case_insensitive
	@Test
	public void forNameCaseInsensitive() {
		assertEquals(StepId.INIT_START_GAME, factory.forName("INITSTARTGAME"));
		assertEquals(StepId.BLOCK_ROLL, factory.forName("BlockRoll"));
	}

	// rust: for_name_old_name_init_activation
	@Test
	public void forNameOldNameInitActivation() {
		assertEquals(StepId.INIT_ACTIVATION, factory.forName("recoverFromGaze"));
	}

	// rust: for_name_old_name_remove_target_selection_state
	@Test
	public void forNameOldNameRemoveTargetSelectionState() {
		assertEquals(StepId.REMOVE_TARGET_SELECTION_STATE, factory.forName("removeBlitzState"));
	}

	// rust: for_name_report_stab_injury_old_name
	@Test
	public void forNameReportStabInjury() {
		assertEquals(StepId.REPORT_STAB_INJURY, factory.forName("reportInjury"));
	}

	// rust: for_name_unknown_returns_none
	@Test
	public void forNameUnknownReturnsNull() {
		assertNull(factory.forName("notAStep"));
	}

	// rust: name_for_block_roll_roundtrip
	@Test
	public void nameForBlockRollRoundtrip() {
		assertEquals(StepId.BLOCK_ROLL, factory.forName(StepId.BLOCK_ROLL.getName()));
	}

	// rust: name_for_init_start_game_roundtrip
	@Test
	public void nameForInitStartGameRoundtrip() {
		assertEquals(StepId.INIT_START_GAME, factory.forName(StepId.INIT_START_GAME.getName()));
	}
}
