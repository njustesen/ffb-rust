package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.ReRolledActions;
import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.common.Catch;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class CatchSkillTest {

    private Catch skill;

    @BeforeEach
    void setUp() {
        skill = new Catch();
        skill.postConstruct();
    }

    @Test
    void name_is_Catch() {
        assertEquals("Catch", skill.getName());
    }

    @Test
    void category_is_agility() {
        assertEquals(SkillCategory.AGILITY, skill.getCategory());
    }

    @Test
    void has_no_named_properties() {
        assertTrue(skill.getSkillProperties().isEmpty(),
            "Catch uses a ReRollSource rather than NamedProperties");
    }

    @Test
    void has_catch_reroll_source() {
        assertNotNull(skill.getRerollSource(ReRolledActions.CATCH),
            "Catch must register a reroll source for failed catch rolls");
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = Catch.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation, "Catch must have a RulesCollection annotation");
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
