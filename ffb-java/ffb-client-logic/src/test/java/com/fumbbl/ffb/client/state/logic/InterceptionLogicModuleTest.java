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

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/interception_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// SKIPPED (with reasons):
// - set_up_clears_interception_skill: Rust asserts directly on the private
//   `interception_skill` field. Java's `interceptionSkill` field has no getter, so this
//   is not observable without reflection.
// - is_interceptor_is_always_false: Rust's `is_interceptor` is a documented conservative
//   gap that always returns `false` (UtilPassing.findInterceptors has no callable Rust
//   equivalent). Java's real `isInterceptor` computes real logic via
//   `UtilPassing.findInterceptors(game, game.getThrower(), game.getPassCoordinate())`
//   against a live, populated Game -- not equivalent behavior, and out of scope
//   (un-mockable engine state).
// - player_peek_resets_since_no_player_is_ever_an_interceptor: depends on the same
//   live-Game-backed `isInterceptor` as above -- out of scope.
//
// PORTED (not from an explicit Rust #[test], but matching the general getId()/should_panic
// categories that are safe to port for every LogicModule subclass):
// - getId() returns ClientStateId.INTERCEPTION
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class InterceptionLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	FantasyFootballClient client;

	@Test
	void getIdReturnsInterception() {
		InterceptionLogicModule module = new InterceptionLogicModule(client);

		assertEquals(ClientStateId.INTERCEPTION, module.getId());
	}

	@Test
	void actionContextPanics() {
		InterceptionLogicModule module = new InterceptionLogicModule(client);
		ActingPlayer actingPlayer = new ActingPlayer(null);

		assertThrows(UnsupportedOperationException.class, () -> module.actionContext(actingPlayer));
	}
}
