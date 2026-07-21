package com.fumbbl.ffb.client.report.bb2025;

import com.fumbbl.ffb.FactoryType.Factory;
import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PassingDistance;
import com.fumbbl.ffb.client.StatusReport;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.client.report.mixed.NervesOfSteelMessage;
import com.fumbbl.ffb.factory.PassModifierFactory;
import com.fumbbl.ffb.mechanics.Mechanic;
import com.fumbbl.ffb.mechanics.PassResult;
import com.fumbbl.ffb.mechanics.bb2025.PassMechanic;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.modifiers.ModifierType;
import com.fumbbl.ffb.modifiers.PassModifier;
import com.fumbbl.ffb.report.IReport;
import com.fumbbl.ffb.report.ReportId;
import com.fumbbl.ffb.report.mixed.ReportPassRoll;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.BDDMockito.given;
import static org.mockito.Mockito.doAnswer;
import static org.mockito.Mockito.spy;

/**
 * Rust's {@code missing_thrower_renders_nothing} test has no 1:1 Java analog: the bb2025
 * Java handler (unlike the Rust port) does not early-return when {@code getPlayerById}
 * yields no player - it unconditionally calls {@code mechanic.formatReportRoll(roll, thrower)},
 * which dereferences {@code thrower} and NPEs on a null player. That test is skipped.
 */
class PassRollMessageTest extends ReportMessageTestBase {

	@SuppressWarnings("rawtypes")
	@Mock
	private Player thrower;

	@SuppressWarnings("rawtypes")
	@Mock
	private Player catcher;

	@Mock
	private PassModifierFactory pmf;

	private final PassModifier nervesOfSteelModifier = new PassModifier("Nerves of Steel", 0, ModifierType.REGULAR);

	private void stubMechanic() {
		given(game.getRules().getFactory(Factory.MECHANIC).forName(Mechanic.Type.PASS.name()))
			.willReturn(new PassMechanic());
	}

	private void stubPassModifierFactory() {
		given(game.getFactory(Factory.PASS_MODIFIER)).willReturn(pmf);
	}

	private void stubThrower() {
		given(game.getPlayerById("t1")).willReturn(thrower);
		given(thrower.getName()).willReturn("Thrower");
		given(game.getTeamHome().hasPlayer(thrower)).willReturn(true);
	}

	@Test
	public void accuratePassReportsSuccessAndNeededRoll() {
		stubThrower();
		stubMechanic();
		stubPassModifierFactory();

		ReportPassRoll report = new ReportPassRoll("t1", 4, 3, false, null, PassingDistance.SHORT_PASS, false, PassResult.ACCURATE);
		List<Run> runs = render(new PassRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("passes the ball.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Succeeded on a roll of 3+")));
	}

	@Test
	public void fumbleReportsFailureAndNeededRoll() {
		stubThrower();
		stubMechanic();
		stubPassModifierFactory();

		ReportPassRoll report = new ReportPassRoll("t1", 1, 3, false, null, PassingDistance.SHORT_PASS, false, PassResult.FUMBLE);
		List<Run> runs = render(new PassRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("fumbles the ball.")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("Roll a 3+ to succeed")));
	}

	@Test
	public void reRolledPassSkipsActionLineAndNeededRoll() {
		stubThrower();
		stubMechanic();
		stubPassModifierFactory();

		ReportPassRoll report = new ReportPassRoll("t1", 4, 3, true, null, PassingDistance.SHORT_PASS, false, PassResult.ACCURATE);
		List<Run> runs = render(new PassRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertFalse(texts.stream().anyMatch(t -> t != null && t.contains("passes the ball to")));
		assertFalse(texts.stream().anyMatch(t -> t != null && t.contains("Succeeded on a roll of")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("passes the ball.")));
	}

	@Test
	public void bombAtCatcherReportsBombWording() {
		stubThrower();
		stubMechanic();
		stubPassModifierFactory();

		FieldCoordinate coordinate = new FieldCoordinate(1, 1);
		given(game.getPassCoordinate()).willReturn(coordinate);
		given(game.getFieldModel().getPlayer(coordinate)).willReturn(catcher);
		given(catcher.getName()).willReturn("Catcher");
		given(game.getTeamHome().hasPlayer(catcher)).willReturn(false);

		ReportPassRoll report = new ReportPassRoll("t1", 4, 3, false, null, PassingDistance.SHORT_PASS, true, PassResult.ACCURATE);
		List<Run> runs = render(new PassRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("throws a bomb at ")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("throws the bomb successfully.")));
	}

	@Test
	public void reportIdIsPassRoll() {
		assertEquals(ReportId.PASS_ROLL.getKey(), new PassRollMessage().getKey());
	}

	@Test
	public void nervesOfSteelModifierRendersSubReport() {
		stubThrower();
		stubMechanic();
		stubPassModifierFactory();
		given(pmf.forName("Nerves of Steel")).willReturn(nervesOfSteelModifier);
		given(thrower.getId()).willReturn("t1");
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		stubSubReportRendering();

		ReportPassRoll report = new ReportPassRoll("t1", 4, 3, false, new PassModifier[]{nervesOfSteelModifier},
			PassingDistance.SHORT_PASS, false, PassResult.ACCURATE);
		List<Run> runs = render(new PassRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("is using Nerves of Steel to")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("pass the ball.")));
	}

	@Test
	public void nervesOfSteelModifierWithBombRendersThrowBombText() {
		stubThrower();
		stubMechanic();
		stubPassModifierFactory();
		given(pmf.forName("Nerves of Steel")).willReturn(nervesOfSteelModifier);
		given(thrower.getId()).willReturn("t1");
		given(game.getActingPlayer().getPlayer()).willReturn(thrower);
		stubSubReportRendering();

		ReportPassRoll report = new ReportPassRoll("t1", 4, 3, false, new PassModifier[]{nervesOfSteelModifier},
			PassingDistance.SHORT_PASS, true, PassResult.ACCURATE);
		List<Run> runs = render(new PassRollMessage(), report);
		List<String> texts = runs.stream().map(r -> r.text).collect(java.util.stream.Collectors.toList());

		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("is using Nerves of Steel to")));
		assertTrue(texts.stream().anyMatch(t -> t != null && t.contains("throw the bomb.")));
	}

	/**
	 * Java: {@code statusReport.report(new ReportNervesOfSteel(...))} looks up the renderer
	 * from a registry populated by {@code StatusReport.init(GameOptions)}, which our fixture
	 * never calls. We spy the real {@code statusReport} instance and redirect
	 * {@code report(IReport)} straight to {@link NervesOfSteelMessage#renderMessage}, mirroring
	 * what the real registry would have dispatched to.
	 */
	private void stubSubReportRendering() {
		StatusReport spyStatusReport = spy(statusReport);
		doAnswer(invocation -> {
			IReport r = invocation.getArgument(0);
			NervesOfSteelMessage message = new NervesOfSteelMessage();
			message.setStatusReport(spyStatusReport);
			message.renderMessage(game, r);
			return null;
		}).when(spyStatusReport).report(any(IReport.class));
		statusReport = spyStatusReport;
	}
}
