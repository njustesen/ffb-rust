package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.Dauntless;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class DauntlessSkillTest {

    private Dauntless skill;

    @BeforeEach
    void setUp() {
        skill = new Dauntless();
        skill.postConstruct();
    }

    @Test
    void name_is_Dauntless() {
        assertEquals("Dauntless", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_can_roll_to_match_opponents_strength_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canRollToMatchOpponentsStrength));
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = Dauntless.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
