package com.fumbbl.ffb.client.report.mixed;

import com.fumbbl.ffb.client.ParagraphStyle;
import com.fumbbl.ffb.client.TextStyle;
import com.fumbbl.ffb.client.report.ReportMessageTestBase;
import com.fumbbl.ffb.model.skill.Skill;
import com.fumbbl.ffb.report.mixed.ReportModifiedDodgeResultSuccessful;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.BDDMockito.given;

class ModifiedDodgeResultSuccessfulMessageTest extends ReportMessageTestBase {

	@Mock
	private Skill skill;

	@Test
	public void rendersSkillName() {
		given(skill.getName()).willReturn("Dodge");

		ReportModifiedDodgeResultSuccessful report = new ReportModifiedDodgeResultSuccessful(skill);
		List<Run> runs = render(new ModifiedDodgeResultSuccessfulMessage(), report);

		assertEquals("Using Dodge would result in a successful dodge", runs.get(0).text);
		assertEquals(TextStyle.EXPLANATION, runs.get(0).textStyle);
	}

	@Test
	public void indentOffsetByOne() {
		given(skill.getName()).willReturn("Dodge");
		statusReport.setIndent(2);

		ReportModifiedDodgeResultSuccessful report = new ReportModifiedDodgeResultSuccessful(skill);
		List<Run> runs = render(new ModifiedDodgeResultSuccessfulMessage(), report);

		assertEquals(ParagraphStyle.INDENT_3, runs.get(0).paragraphStyle);
	}

	// Rust's "missing_skill_renders_empty_name" test has no Java analog: the Java handler
	// calls report.getSkill().getName() unconditionally, which throws NPE on a null skill
	// (this is explicitly noted in the Rust source as a knowingly-preserved Java quirk).
}
