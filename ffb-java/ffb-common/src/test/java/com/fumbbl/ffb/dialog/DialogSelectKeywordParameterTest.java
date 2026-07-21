package com.fumbbl.ffb.dialog;

import com.fumbbl.ffb.KeywordChoiceMode;
import com.fumbbl.ffb.model.Keyword;

import org.junit.jupiter.api.Test;

import java.util.Arrays;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Mirrors the surviving Rust tests in
 * crates/ffb-model/src/dialog/dialog_select_keyword_parameter.rs for
 * {@link DialogSelectKeywordParameter}.
 *
 * <p>The Rust test uses the raw keyword string "PASS"; the Java model stores
 * typed {@link Keyword} values, so a real keyword ({@link Keyword#BLITZER}) is
 * used instead. This does not affect what the test exercises: transform()
 * hard-resets minSelect/maxSelect to 1 while passing player id and keywords
 * through unchanged.
 */
public class DialogSelectKeywordParameterTest {

	@Test
	public void transformResetsMinMaxSelectToOne() {
		DialogSelectKeywordParameter p = new DialogSelectKeywordParameter("p1", Arrays.asList(Keyword.BLITZER),
			KeywordChoiceMode.GETTING_EVEN, 2, 5);
		DialogSelectKeywordParameter transformed = (DialogSelectKeywordParameter) p.transform();
		assertEquals(1, transformed.getMinSelect());
		assertEquals(1, transformed.getMaxSelect());
		assertEquals("p1", transformed.getPlayerId());
		assertEquals(Arrays.asList(Keyword.BLITZER), transformed.getKeywords());
	}

}
