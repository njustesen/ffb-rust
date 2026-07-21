package com.fumbbl.ffb.model.change;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in crates/ffb-model/src/model/change/model_change_list.rs for {@link ModelChangeList}.
 */
public class ModelChangeListTest {

	@Test
	public void addIncreasesLen() {
		ModelChangeList list = new ModelChangeList();
		list.add(new ModelChange(ModelChangeId.GAME_SET_HOME_PLAYING, null, null));
		assertEquals(1, list.size());
	}

}
