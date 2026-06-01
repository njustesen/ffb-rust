package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.FoulAppearance;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class FoulAppearanceSkillTest {

    private FoulAppearance skill;

    @BeforeEach
    void setUp() {
        skill = new FoulAppearance();
        skill.postConstruct();
    }

    @Test
    void name_is_Foul_Appearance() {
        assertEquals("Foul Appearance", skill.getName());
    }

    @Test
    void category_is_mutation() {
        assertEquals(SkillCategory.MUTATION, skill.getCategory());
    }

    @Test
    void has_force_roll_before_being_blocked_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.forceRollBeforeBeingBlocked));
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = FoulAppearance.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
