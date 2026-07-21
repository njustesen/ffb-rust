package com.fumbbl.ffb.factory;

import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.inducement.InducementType;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/factory/inducement_type_factory.rs
 * for {@link InducementTypeFactory}.
 * <p>
 * The Rust {@code InducementType.id} corresponds to the Java {@link InducementType#getName()}.
 */
public class InducementTypeFactoryTest {

	private static InducementTypeFactory factory() {
		return NetCommandTestUtil.gameSource().getFactory(Factory.INDUCEMENT_TYPE);
	}

	@Test
	public void forNameFindsExtraTeamTraining() {
		InducementType found = factory().forName("extraTeamTraining");
		assertEquals("extraTeamTraining", found.getName());
	}

	@Test
	public void forNameIsCaseInsensitive() {
		assertNotNull(factory().forName("EXTRATEAMTRAINING"));
	}

	@Test
	public void forNameReturnsNoneForUnknown() {
		assertNull(factory().forName("unknown"));
	}

	@Test
	public void allTypesAreSortedById() {
		List<InducementType> all = factory().allTypes();
		List<String> ids = new ArrayList<>();
		for (InducementType type : all) {
			ids.add(type.getName());
		}
		List<String> sortedIds = new ArrayList<>(ids);
		sortedIds.sort(Comparator.naturalOrder());
		assertEquals(sortedIds, ids);
	}
}
