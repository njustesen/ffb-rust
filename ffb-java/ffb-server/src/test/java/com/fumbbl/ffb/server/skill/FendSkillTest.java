package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.common.Fend;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class FendSkillTest {

    private Fend skill;

    @BeforeEach
    void setUp() {
        skill = new Fend();
        skill.postConstruct();
    }

    @Test
    void name_is_Fend() {
        assertEquals("Fend", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_prevent_opponent_following_up_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.preventOpponentFollowingUp));
    }

    @Test
    void is_common_across_all_editions() {
        RulesCollection annotation = Fend.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.COMMON, annotation.value());
    }
}
