package com.fumbbl.ffb.inducement;

import org.junit.jupiter.api.Test;

import java.util.Collections;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/inducement/inducement_collection.rs for {@link InducementCollection}.
 * The Rust root collection carries only the base types; in Java the base class is abstract and folds in
 * {@link InducementCollection#getSubTypes()}, so an empty-subtype subclass exposes exactly the base set.
 */
public class InducementCollectionTest {

	private static InducementCollection baseCollection() {
		return new InducementCollection() {
			@Override
			protected Set<InducementType> getSubTypes() {
				return Collections.emptySet();
			}
		};
	}

	@Test
	public void baseCollectionHasFiveTypes() {
		InducementCollection c = baseCollection();
		assertEquals(5, c.getTypes().size());
	}

	@Test
	public void hasExtraTeamTraining() {
		InducementCollection c = baseCollection();
		assertTrue(c.getTypes().stream().anyMatch(t -> "extraTeamTraining".equals(t.getName())));
	}
}
