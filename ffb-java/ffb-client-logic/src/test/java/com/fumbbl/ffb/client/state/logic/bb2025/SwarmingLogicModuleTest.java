package com.fumbbl.ffb.client.state.logic.bb2025;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.Position;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.Collections;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/bb2025/swarming_logic_module.rs
// (Rust: mod tests). SwarmingLogicModule extends SetupLogicModule, whose constructor only calls
// super(client) (no eager plugin factory lookup), so a plain explicitly-wired Game/FieldModel mock
// suffices without needing the MoveLogicModule-style plugin stubbing.
//
// SKIPPED (with reasons):
// - action_context_panics_like_setup_logic_module: `actionContext(ActingPlayer)` is not overridden
//   in `SwarmingLogicModule` (inherited unchanged from `SetupLogicModule`, itself declared
//   `protected` in the base `com.fumbbl.ffb.client.state.logic` package). Since it is neither
//   re-declared in the `bb2025` package nor is this test class a subclass, Java protected-access
//   rules make it inaccessible from here.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class SwarmingLogicModuleTest {

	@Mock
	FantasyFootballClient client;

	@Mock
	Game game;

	@Mock
	FieldModel fieldModel;

	@Mock
	@SuppressWarnings("rawtypes")
	Player player;

	@Mock
	Position position;

	@BeforeEach
	void setUp() {
		when(client.getGame()).thenReturn(game);
		when(game.getFieldModel()).thenReturn(fieldModel);
	}

	@Test
	void squareHasSwarmingPlayerFalseWithoutGame() {
		SwarmingLogicModule module = new SwarmingLogicModule(client);
		assertFalse(module.squareHasSwarmingPlayer(new FieldCoordinate(1, 1)));
	}

	@Test
	@SuppressWarnings("unchecked")
	void squareHasSwarmingPlayerFalseForNonLinemanPlayer() {
		when(fieldModel.getPlayer(new FieldCoordinate(3, 3))).thenReturn(player);
		when(player.getPosition()).thenReturn(position);
		when(position.getKeywords()).thenReturn(Collections.emptyList());
		SwarmingLogicModule module = new SwarmingLogicModule(client);
		assertFalse(module.squareHasSwarmingPlayer(new FieldCoordinate(3, 3)));
	}

	@Test
	@SuppressWarnings("unchecked")
	void squareHasSwarmingPlayerTrueForLinemanPlayer() {
		when(fieldModel.getPlayer(new FieldCoordinate(3, 3))).thenReturn(player);
		when(player.getPosition()).thenReturn(position);
		when(position.getKeywords()).thenReturn(List.of(com.fumbbl.ffb.model.Keyword.LINEMAN));
		SwarmingLogicModule module = new SwarmingLogicModule(client);
		assertTrue(module.squareHasSwarmingPlayer(new FieldCoordinate(3, 3)));
	}

	@Test
	void squareIsValidForSwarmingTrueForEmptyHalfHomeSquare() {
		SwarmingLogicModule module = new SwarmingLogicModule(client);
		assertTrue(module.squareIsValidForSwarming(new FieldCoordinate(1, 1)));
	}

	@Test
	void squareIsValidForSwarmingFalseWhenOccupied() {
		when(fieldModel.getPlayer(new FieldCoordinate(2, 2))).thenReturn(player);
		SwarmingLogicModule module = new SwarmingLogicModule(client);
		assertFalse(module.squareIsValidForSwarming(new FieldCoordinate(2, 2)));
	}

	@Test
	void squareIsValidForSwarmingFalseOutsideHalfHomeAndNotBox() {
		SwarmingLogicModule module = new SwarmingLogicModule(client);
		assertFalse(module.squareIsValidForSwarming(new FieldCoordinate(20, 5)));
	}
}
