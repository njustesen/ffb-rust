package com.fumbbl.ffb.client.state.logic.mixed;

import com.fumbbl.ffb.ClientStateId;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerChoiceMode;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.net.ClientCommunication;
import com.fumbbl.ffb.client.state.logic.ClientAction;
import com.fumbbl.ffb.client.state.logic.interaction.ActionContext;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.Player;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/mixed/raiding_party_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// All Rust tests are ported: RaidingPartyLogicModule extends LogicModule directly (no
// MoveLogicModule/BlockLogicModule plugin-factory constructor chain), and every method it
// overrides either ignores game state entirely (actionContext) or only compares against
// client.getGame()'s deep-stub defaults (null Player/FieldModel lookups), which coincide
// exactly with the "without game" scenarios exercised in Rust.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class RaidingPartyLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	FantasyFootballClient client;

	@Mock
	Player<?> player;

	// NOTE (test equalization): getIdIsRaidingParty pruned — trivial getter, no Rust twin.

	@Test
	void availableActionsIsRaidingPartyOnly() {
		RaidingPartyLogicModule module = new RaidingPartyLogicModule(client);
		Set<ClientAction> actions = module.availableActions();
		assertEquals(1, actions.size());
		assertTrue(actions.contains(ClientAction.RAIDING_PARTY));
	}

	@Test
	void actionContextAlwaysAddsRaidingParty() {
		RaidingPartyLogicModule module = new RaidingPartyLogicModule(client);
		ActingPlayer actingPlayer = new ActingPlayer(null);
		ActionContext ctx = module.actionContext(actingPlayer);
		assertEquals(List.of(ClientAction.RAIDING_PARTY), ctx.getActions());
	}

	@Test
	void playerPeekInvalidWithoutGame() {
		RaidingPartyLogicModule module = new RaidingPartyLogicModule(client);
		InteractionResult result = module.playerPeek(player);
		assertEquals(InteractionResult.invalid().getKind(), result.getKind());
	}

	@Test
	void playerInteractionIgnoresWithoutGame() {
		RaidingPartyLogicModule module = new RaidingPartyLogicModule(client);
		InteractionResult result = module.playerInteraction(player);
		assertEquals(InteractionResult.ignore().getKind(), result.getKind());
	}

	@Test
	void fieldInteractionIgnoresWithoutMoveSquare() {
		RaidingPartyLogicModule module = new RaidingPartyLogicModule(client);
		InteractionResult result = module.fieldInteraction(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.ignore().getKind(), result.getKind());
	}

	@Test
	void fieldPeekInvalidWithoutMoveSquare() {
		RaidingPartyLogicModule module = new RaidingPartyLogicModule(client);
		InteractionResult result = module.fieldPeek(new FieldCoordinate(3, 3));
		assertEquals(InteractionResult.invalid().getKind(), result.getKind());
	}

	@Test
	void performAvailableActionSendsPlayerChoice() {
		RaidingPartyLogicModule module = new RaidingPartyLogicModule(client);
		ClientCommunication communication = client.getCommunication();

		module.performAvailableAction(player, ClientAction.RAIDING_PARTY);

		verify(communication).sendPlayerChoice(PlayerChoiceMode.RAIDING_PARTY, null);
	}
}
