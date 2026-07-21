package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.ReRollOptions;
import com.fumbbl.ffb.ReRollProperty;
import com.fumbbl.ffb.inducement.InducementType;
import com.fumbbl.ffb.net.NetCommandTestUtil;

import com.eclipsesource.json.JsonValue;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_re_roll_regeneration_multiple_parameter.rs
 * for {@link DialogReRollRegenerationMultipleParameter}.
 */
public class DialogReRollRegenerationMultipleParameterTest {

	@Test
	public void emptyPlayerIdsEdgeCase() {
		DialogReRollRegenerationMultipleParameter p =
			new DialogReRollRegenerationMultipleParameter(new ArrayList<>(), (InducementType) null);
		assertTrue(p.getPlayerIds().isEmpty());
		assertNull(p.getInducementType());
	}

	@Test
	public void reRollOptionsFieldIsPreserved() {
		ReRollOptions opts = new ReRollOptions(Arrays.asList(ReRollProperty.TRR), null);
		DialogReRollRegenerationMultipleParameter p =
			new DialogReRollRegenerationMultipleParameter(Arrays.asList("p1"), Arrays.asList(opts));
		assertEquals(1, p.getReRollOptions().size());
		assertTrue(p.getReRollOptions().get(0).canActuallyReRoll());

		JsonValue json = p.toJsonValue();
		DialogReRollRegenerationMultipleParameter back = (DialogReRollRegenerationMultipleParameter)
			new DialogReRollRegenerationMultipleParameter().initFrom(NetCommandTestUtil.gameSource(), json);
		assertEquals(1, back.getReRollOptions().size());
	}

}
