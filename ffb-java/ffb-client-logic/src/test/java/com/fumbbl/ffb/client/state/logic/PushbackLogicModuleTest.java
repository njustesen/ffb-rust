package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.model.ActingPlayer;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/pushback_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// SKIPPED (with reasons):
// - field_interaction_ignores_when_no_pushback_square
// - field_interaction_sends_pushback_when_square_found
// - field_peek_selects_matching_unlocked_home_choice_square
// - field_peek_deselects_previously_selected_square_on_miss
// - pushback_to_finds_square_by_direction
//   All five require driving fieldInteraction()/fieldPeek()/pushbackTo() against a real,
//   populated Game/FieldModel with pushback squares and player coordinates (as the Rust
//   test does with a real ffb-model Game). That is exactly the "live game state to drive
//   fieldInteraction()" scope excluded by this porting pass.
//
// PORTED (not from an explicit Rust #[test], but matching the getId() category that is
// safe to port for every LogicModule subclass):
// - getId() returns ClientStateId.PUSHBACK
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class PushbackLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	FantasyFootballClient client;

	@Test
	void getIdReturnsPushback() {
		PushbackLogicModule module = new PushbackLogicModule(client);

		assertEquals(ClientStateId.PUSHBACK, module.getId());
	}

	@Test
	void actionContextPanics() {
		PushbackLogicModule module = new PushbackLogicModule(client);
		ActingPlayer actingPlayer = new ActingPlayer(null);

		assertThrows(UnsupportedOperationException.class, () -> module.actionContext(actingPlayer));
	}
}
