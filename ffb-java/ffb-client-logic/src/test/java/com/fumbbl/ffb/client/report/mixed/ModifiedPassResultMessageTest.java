package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.mechanics.PassResult;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.mixed.ReportModifiedPassResult;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.BDDMockito.given;

class ModifiedPassResultMessageTest extends ReportMessageTestBase {

	@Mock
	private Skill skill;

	@Test
	public void rendersSkillAndPassResult() {
		given(skill.getName()).willReturn("Pass");

		ReportModifiedPassResult report = new ReportModifiedPassResult(skill, PassResult.FUMBLE);
		List<Run> runs = render(new ModifiedPassResultMessage(), report);

		assertEquals("Using Pass would change the result to FUMBLE", runs.get(0).text);
		assertEquals(TextStyle.EXPLANATION, runs.get(0).textStyle);
	}

	@Test
	public void differentPassResultValue() {
		given(skill.getName()).willReturn("Pass");

		ReportModifiedPassResult report = new ReportModifiedPassResult(skill, PassResult.INACCURATE);
		List<Run> runs = render(new ModifiedPassResultMessage(), report);

		assertTrue(runs.get(0).text.endsWith("INACCURATE"));
	}

	// Rust's "missing_skill_renders_empty_name" test has no Java analog: the Java handler
	// calls report.getSkill().getName() unconditionally, which throws NPE on a null skill.
	// Also note: Rust's report.get_pass_result() is a loose String field ("Fumble",
	// "Inaccurate"), while Java's PassResult is a real enum whose getName() returns the
	// SCREAMING_SNAKE_CASE constant name (e.g. "FUMBLE"), so the expected text differs
	// from the Rust literal but the mechanic (skill name + pass result name) is preserved.
}
