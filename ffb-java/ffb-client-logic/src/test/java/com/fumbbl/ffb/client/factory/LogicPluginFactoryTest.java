package com.fumbbl.ffb.client.factory;

import com.fumbbl.ffb.client.state.logic.plugin.LogicPlugin;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Ported from {@code ffb-rust/crates/ffb-client/src/client/factory/logic_plugin_factory.rs}
 * against the real {@link LogicPluginFactory}.
 *
 * <p>Rust tests pruned rather than ported (kept the suites 1:1): the four {@code initialize_*} /
 * {@code for_name_resolves_by_type_name} / {@code initialize_clears_previous_registrations} tests
 * all call {@code initialize(game)}, which runs a classpath {@code Scanner} over a live
 * {@code game.getOptions()} to discover LogicPlugin implementations — fixture-inexpressible with
 * targeted mocks.
 */
class LogicPluginFactoryTest {

	// rust: for_type_and_for_name_are_empty_before_initialize
	@Test
	void forTypeAndForNameAreEmptyBeforeInitialize() {
		LogicPluginFactory factory = new LogicPluginFactory();
		assertNull(factory.forType(LogicPlugin.Type.BASE));
		assertNull(factory.forName("BASE"));
	}
}
