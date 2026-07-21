package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.ReRollOptions;
import com.fumbbl.ffb.ReRollProperty;
import com.fumbbl.ffb.factory.IFactorySource;
import com.fumbbl.ffb.inducement.InducementType;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.net.NetCommandTestUtil;
import com.fumbbl.ffb.option.GameOptionId;
import com.fumbbl.ffb.option.GameOptionString;

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
			new DialogReRollRegenerationMultipleParameter().initFrom(bb2025Source(), json);
		assertEquals(1, back.getReRollOptions().size());
	}

	// The ReRollProperty factory (RE_ROLL_PROPERTY) is registered only for BB2025 rules,
	// so the JSON round-trip needs a BB2025 game context to resolve the property names.
	private static IFactorySource bb2025Source() {
		IFactorySource app = NetCommandTestUtil.applicationSource();
		Game game = new Game(app, app.getFactoryManager());
		GameOptionString rules = new GameOptionString(GameOptionId.RULESVERSION);
		rules.setValue("BB2025");
		game.getOptions().addOption(rules);
		game.initializeRules();
		return game.getRules();
	}

}
