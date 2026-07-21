package com.fumbbl.ffb.client.report.bb2020;

import com.fumbbl.ffb.FactoryType;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PassingDistance;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.factory.MechanicsFactory;
import com.fumbbl.ffb.factory.PassModifierFactory;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.PassMechanic;
import com.fumbbl.ffb.mechanics.PassResult;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.report.mixed.ReportPassRoll;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class PassRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player catcher;

	@Mock
	private MechanicsFactory mechanicsFactory;

	@Mock
	private PassMechanic mechanic;

	@Mock
	private PassModifierFactory passModifierFactory;

	@BeforeEach
	public void setUpMechanicAndThrower() {
		// Handler casts the forName() RESULT to PassMechanic, so stub the terminal chain call
		// (green idiom); doReturn(...).when(game.getRules()) is unreliable on deep stubs.
		given(game.getRules().getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.PASS.name())).willReturn(mechanic);
		// Handler casts the root getFactory() to PassModifierFactory, so force it via doReturn on game.
		org.mockito.Mockito.doReturn(passModifierFactory).when(game).getFactory(FactoryType.Factory.PASS_MODIFIER);
		given(game.getPlayerById("thrower")).willReturn(thrower);
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
	}

	private List<String> texts(List<Run> runs) {
		return runs.stream().map(r -> r.text).filter(Objects::nonNull).collect(Collectors.toList());
	}

	@Test
	public void accuratePassToCatcher() {
		given(game.getPassCoordinate()).willReturn(new FieldCoordinate(5, 5));
		given(game.getFieldModel().getPlayer(new FieldCoordinate(5, 5))).willReturn(catcher);
		ReportPassRoll report = new ReportPassRoll("thrower", 4, 2, false, null,
			PassingDistance.SHORT_PASS, false, PassResult.ACCURATE, false, null);
		List<Run> runs = render(new PassRollMessage(), report);
		List<String> t = texts(runs);
		assertTrue(t.stream().anyMatch(s -> s.contains("passes the ball to ")));
		assertTrue(t.stream().anyMatch(s -> s.contains(" passes the ball.")));
		assertTrue(t.stream().anyMatch(s -> s.contains("Succeeded on a roll of 2+")));
	}

	@Test
	public void bombFumble() {
		given(game.getPassCoordinate()).willReturn(new FieldCoordinate(5, 5));
		given(game.getFieldModel().getPlayer(new FieldCoordinate(5, 5))).willReturn(catcher);
		ReportPassRoll report = new ReportPassRoll("thrower", 1, 2, false, null,
			PassingDistance.SHORT_PASS, true, PassResult.FUMBLE, false, null);
		List<Run> runs = render(new PassRollMessage(), report);
		List<String> t = texts(runs);
		assertTrue(t.stream().anyMatch(s -> s.contains("throws a bomb at")));
		assertTrue(t.stream().anyMatch(s -> s.contains("fumbles the bomb.")));
		assertTrue(t.stream().anyMatch(s -> s.contains("Roll a 2+ to succeed")));
	}

	@Test
	public void hailMaryPassInaccurateCountsAsSuccess() {
		given(game.getPassCoordinate()).willReturn(null);
		ReportPassRoll report = new ReportPassRoll("thrower", 2, 2, false, null,
			null, false, PassResult.INACCURATE, true, null);
		List<Run> runs = render(new PassRollMessage(), report);
		List<String> t = texts(runs);
		assertTrue(t.stream().anyMatch(s -> s.contains("throws a Hail Mary pass:")));
		assertTrue(t.stream().anyMatch(s -> s.contains(" passes the ball.")));
	}

	@Test
	public void reRolledSkipsIntroAndNeededRoll() {
		given(game.getPassCoordinate()).willReturn(new FieldCoordinate(5, 5));
		given(game.getFieldModel().getPlayer(new FieldCoordinate(5, 5))).willReturn(catcher);
		ReportPassRoll report = new ReportPassRoll("thrower", 4, 2, true, null,
			PassingDistance.SHORT_PASS, false, PassResult.ACCURATE, false, null);
		List<Run> runs = render(new PassRollMessage(), report);
		List<String> t = texts(runs);
		assertFalse(t.stream().anyMatch(s -> s.contains("passes the ball to ")));
		assertFalse(t.stream().anyMatch(s -> s.contains("Succeeded on a roll")));
	}

	@Test
	public void emptyFieldPass() {
		given(game.getPassCoordinate()).willReturn(new FieldCoordinate(10, 10));
		// Deep-stub would auto-vivify a non-null catcher; force empty field.
		given(game.getFieldModel().getPlayer(new FieldCoordinate(10, 10))).willReturn(null);
		ReportPassRoll report = new ReportPassRoll("thrower", 4, 2, false, null,
			PassingDistance.SHORT_PASS, false, PassResult.ACCURATE, false, null);
		List<Run> runs = render(new PassRollMessage(), report);
		List<String> t = texts(runs);
		assertTrue(t.stream().anyMatch(s -> s.contains("passes the ball to an empty field:")));
	}
}
