package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.DivingCatch;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class DivingCatchSkillTest {

    private DivingCatch skill;

    @BeforeEach
    void setUp() {
        skill = new DivingCatch();
        skill.postConstruct();
    }

    @Test
    void name_is_Diving_Catch() {
        assertEquals("Diving Catch", skill.getName());
    }

    @Test
    void category_is_agility() {
        assertEquals(SkillCategory.AGILITY, skill.getCategory());
    }

    @Test
    void has_can_attempt_catch_in_adjacent_squares_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canAttemptCatchInAdjacentSquares));
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = DivingCatch.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
