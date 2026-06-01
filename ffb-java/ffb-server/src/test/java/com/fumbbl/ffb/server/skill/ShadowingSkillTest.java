package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Shadowing;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ShadowingSkillTest {

    private Shadowing skill;

    @BeforeEach
    void setUp() {
        skill = new Shadowing();
        skill.postConstruct();
    }

    @Test
    void name_is_Shadowing() {
        assertEquals("Shadowing", skill.getName());
    }

    @Test
    void category_is_general() {
        assertEquals(SkillCategory.GENERAL, skill.getCategory());
    }

    @Test
    void has_can_follow_player_leaving_tacklezones_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canFollowPlayerLeavingTacklezones));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Shadowing.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
