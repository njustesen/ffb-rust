package com.fumbbl.ffb.server.factory;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.fixture.GameFixture;
import com.fumbbl.ffb.server.model.change.ChompRemovalObserver;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirror of ffb-rust crates/ffb-engine/src/model/change/observer_factory.rs tests. The
 * Scanner-populated observer set is empty until initialize(); BB2025 registers the
 * ChompRemovalObserver (@RulesCollection BB2025), BB2016/BB2020 register none. forName is a
 * no-op returning null (observers are matched by class, not name — the Rust for_name accessor is
 * Rust-specific, mirrored here via the getObservers() class membership check). The Rust
 * get_observers_returns_empty_slice_before_init test duplicates newFactoryHasNoObservers — exempt.
 */
public class ObserverFactoryTest {

	private Game gameFor(RulesCollection.Rules rules) {
		GameState gameState = GameFixture.createGameState(3, rules);
		return gameState.getGame();
	}

	// rust: new_factory_is_empty (and get_observers_returns_empty_slice_before_init)
	@Test
	public void newFactoryHasNoObservers() {
		assertTrue(new ObserverFactory().getObservers().isEmpty());
	}

	// rust: initialize_bb2025_registers_chomp_removal_observer
	@Test
	public void initializeBb2025RegistersChompRemovalObserver() {
		ObserverFactory factory = new ObserverFactory();
		factory.initialize(gameFor(RulesCollection.Rules.BB2025));
		assertFalse(factory.getObservers().isEmpty());
		assertTrue(factory.getObservers().stream().anyMatch(o -> o instanceof ChompRemovalObserver));
	}

	// rust: initialize_bb2016_is_empty
	@Test
	public void initializeBb2016IsEmpty() {
		ObserverFactory factory = new ObserverFactory();
		factory.initialize(gameFor(RulesCollection.Rules.BB2016));
		assertTrue(factory.getObservers().isEmpty());
	}

	// rust: initialize_bb2020_is_empty
	@Test
	public void initializeBb2020IsEmpty() {
		ObserverFactory factory = new ObserverFactory();
		factory.initialize(gameFor(RulesCollection.Rules.BB2020));
		assertTrue(factory.getObservers().isEmpty());
	}

	// rust: for_name_returns_none_for_unknown (Java forName is a no-op returning null for any name)
	@Test
	public void forNameAlwaysReturnsNull() {
		ObserverFactory factory = new ObserverFactory();
		factory.initialize(gameFor(RulesCollection.Rules.BB2025));
		assertNull(factory.forName("ChompRemovalObserver"));
		assertNull(factory.forName("NonExistentObserver"));
	}
}
