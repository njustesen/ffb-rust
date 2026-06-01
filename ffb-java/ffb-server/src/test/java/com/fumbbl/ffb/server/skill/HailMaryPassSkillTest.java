package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.HailMaryPass;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class HailMaryPassSkillTest {

    private HailMaryPass skill;

    @BeforeEach
    void setUp() {
        skill = new HailMaryPass();
        skill.postConstruct();
    }

    @Test
    void name_is_Hail_Mary_Pass() {
        assertEquals("Hail Mary Pass", skill.getName());
    }

    @Test
    void category_is_passing() {
        assertEquals(SkillCategory.PASSING, skill.getCategory());
    }

    @Test
    void has_can_pass_to_any_square_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canPassToAnySquare));
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = HailMaryPass.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
