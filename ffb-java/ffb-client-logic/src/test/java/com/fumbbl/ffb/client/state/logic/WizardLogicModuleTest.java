package com.fumbbl.ffb.client.state.logic;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.FieldCoordinateBounds;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.SpecialEffect;
import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.net.ClientCommunication;
import com.fumbbl.ffb.client.state.logic.interaction.InteractionResult;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.RosterPlayer;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.junit.jupiter.MockitoSettings;
import org.mockito.quality.Strictness;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

// Mirrors ffb-rust ffb-client crates/ffb-client/src/client/state/logic/wizard_logic_module.rs
// (Rust: mod tests), one test fn per Rust #[test] that is faithfully reproducible.
//
// NOTE: Java's private `spellAvailable` field has no getter, unlike Rust's test-visible
// `Cell<bool>`. `setUpResetsSpellAvailable` instead observes the flag indirectly through
// `fieldPeek`'s `determineSpecialEffect` branch, which is gated on it, exactly mirroring the
// real effect `setUp()` has on subsequent peeks.
@ExtendWith(MockitoExtension.class)
@MockitoSettings(strictness = Strictness.LENIENT)
class WizardLogicModuleTest {

	@Mock(answer = Answers.RETURNS_DEEP_STUBS)
	FantasyFootballClient client;

	@Test
	void setUpResetsSpellAvailable() {
		WizardLogicModule module = new WizardLogicModule(client);
		when(client.getClientData().getWizardSpell()).thenReturn(SpecialEffect.ZAP);
		FieldCoordinate coordinate = new FieldCoordinate(1, 1);

		InteractionResult before = module.fieldPeek(coordinate);
		assertEquals(InteractionResult.Kind.IGNORE, before.getKind());

		module.setUp();

		InteractionResult after = module.fieldPeek(coordinate);
		assertEquals(InteractionResult.Kind.PERFORM, after.getKind());
	}

	@Test
	void isValidLightningTargetRequiresStandingAwayPlayer() {
		FieldCoordinate coordinate = new FieldCoordinate(5, 5);
		RosterPlayer player = new RosterPlayer();
		player.setId("p1");
		when(client.getGame().getFieldModel().getPlayer(coordinate)).thenReturn((com.fumbbl.ffb.model.Player) player);
		when(client.getGame().getTeamAway().hasPlayer(player)).thenReturn(true);
		when(client.getGame().getFieldModel().getPlayerState(player)).thenReturn(new PlayerState(PlayerState.STANDING));
		WizardLogicModule module = new WizardLogicModule(client);

		assertTrue(module.isValidLightningTarget(coordinate));

		when(client.getGame().getFieldModel().getPlayerState(player)).thenReturn(new PlayerState(PlayerState.PRONE));

		assertFalse(module.isValidLightningTarget(coordinate));
	}

	@Test
	void isValidZapTargetRequiresAwayPlayerPresent() {
		FieldCoordinate coordinate = new FieldCoordinate(6, 6);
		WizardLogicModule module = new WizardLogicModule(client);

		assertFalse(module.isValidZapTarget(coordinate));

		RosterPlayer player = new RosterPlayer();
		player.setId("p1");
		when(client.getGame().getFieldModel().getPlayer(coordinate)).thenReturn((com.fumbbl.ffb.model.Player) player);
		when(client.getGame().getTeamAway().hasPlayer(player)).thenReturn(true);

		assertTrue(module.isValidZapTarget(coordinate));
	}

	@Test
	void isValidFireballTargetChecksAdjacentAndCenterSquares() {
		FieldCoordinate center = new FieldCoordinate(10, 10);
		FieldCoordinate adjacent = new FieldCoordinate(11, 10);
		when(client.getGame().getFieldModel().findAdjacentCoordinates(center, FieldCoordinateBounds.FIELD, 1, true))
			.thenReturn(new FieldCoordinate[] { adjacent });
		RosterPlayer player = new RosterPlayer();
		player.setId("p1");
		when(client.getGame().getFieldModel().getPlayer(adjacent)).thenReturn((com.fumbbl.ffb.model.Player) player);
		when(client.getGame().getTeamAway().hasPlayer(player)).thenReturn(true);
		when(client.getGame().getFieldModel().getPlayerState(player)).thenReturn(new PlayerState(PlayerState.STANDING));
		WizardLogicModule module = new WizardLogicModule(client);

		assertTrue(module.isValidFireballTarget(center));
	}

	@Test
	void handleClickIgnoresWhenNoWizardSpellPending() {
		WizardLogicModule module = new WizardLogicModule(client);

		InteractionResult result = module.fieldInteraction(new FieldCoordinate(1, 1));

		assertEquals(InteractionResult.Kind.IGNORE, result.getKind());
	}

	@Test
	void handleClickSendsSpellAndClearsAvailabilityOnValidTarget() {
		FieldCoordinate coordinate = new FieldCoordinate(3, 3);
		RosterPlayer player = new RosterPlayer();
		player.setId("p1");
		when(client.getGame().getFieldModel().getPlayer(coordinate)).thenReturn((com.fumbbl.ffb.model.Player) player);
		when(client.getGame().getTeamAway().hasPlayer(player)).thenReturn(true);
		when(client.getClientData().getWizardSpell()).thenReturn(SpecialEffect.ZAP);
		WizardLogicModule module = new WizardLogicModule(client);
		ClientCommunication communication = client.getCommunication();

		InteractionResult result = module.fieldInteraction(coordinate);

		assertEquals(InteractionResult.Kind.HANDLED, result.getKind());
		verify(communication).sendWizardSpell(SpecialEffect.ZAP, coordinate);
	}

	@Test
	void handleClickResetsOnInvalidTarget() {
		FieldCoordinate coordinate = new FieldCoordinate(3, 3);
		when(client.getClientData().getWizardSpell()).thenReturn(SpecialEffect.ZAP);
		WizardLogicModule module = new WizardLogicModule(client);

		InteractionResult result = module.fieldInteraction(coordinate);

		assertEquals(InteractionResult.Kind.RESET, result.getKind());
	}

	@Test
	void actionContextPanics() {
		WizardLogicModule module = new WizardLogicModule(client);
		ActingPlayer actingPlayer = new ActingPlayer(null);

		assertThrows(UnsupportedOperationException.class, () -> module.actionContext(actingPlayer));
	}
}
