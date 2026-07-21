package com.fumbbl.ffb.client.handler;

import com.fumbbl.ffb.client.FantasyFootballClient;
import com.fumbbl.ffb.client.dialog.IDialog;
import com.fumbbl.ffb.net.NetCommandId;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.verify;

/**
 * Mirrors Rust {@code client_command_handler_admin_message.rs} tests.
 *
 * <p>Rust left {@code handle_net_command}'s dialog show/close cycle as {@code // java:} notes
 * because {@code DialogInformation}/{@code IDialog} have no Rust equivalent. In the real Java
 * handler, {@code handleNetCommand} unconditionally constructs a real
 * {@code DialogInformation} (a {@code JInternalFrame}-based Swing dialog wired through several
 * chained real UI getters — icon cache, dimension providers, desktop pane, etc.) and calls
 * {@code showDialog(this)}. There is no mockable seam that prevents this real Swing
 * construction, so `handleNetCommand` itself is SKIPPED here (matches the Rust file's own
 * choice to not invent behavior beyond what's translatable). {@code getId()} and
 * {@code dialogClosed(IDialog)} have no such Swing/AWT construction issue and are ported.
 */
@ExtendWith(MockitoExtension.class)
class ClientCommandHandlerAdminMessageTest {

	@Mock
	private FantasyFootballClient client;

	@Mock
	private IDialog dialog;

	private ClientCommandHandlerAdminMessage handler;

	@Test
	void getIdReturnsServerAdminMessage() {
		handler = new ClientCommandHandlerAdminMessage(client);
		assertEquals(NetCommandId.SERVER_ADMIN_MESSAGE, handler.getId());
	}

	@Test
	void dialogClosedHidesTheDialog() {
		handler = new ClientCommandHandlerAdminMessage(client);
		handler.dialogClosed(dialog);
		verify(dialog).hideDialog();
	}

	// Rust `handle_net_command_returns_true_for_matching_command`,
	// `handle_net_command_returns_true_across_modes` and
	// `handle_net_command_is_a_no_op_for_a_mismatched_command_type` SKIPPED: the real Java
	// handleNetCommand unconditionally builds a real DialogInformation (Swing) with no
	// mockable seam (see class javadoc); the mismatched-type case additionally would throw
	// ClassCastException rather than no-op in Java.
}
