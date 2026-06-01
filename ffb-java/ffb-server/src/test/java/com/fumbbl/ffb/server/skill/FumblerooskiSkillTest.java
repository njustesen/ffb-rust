package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.Fumblerooski;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class FumblerooskiSkillTest {

    private Fumblerooski skill;

    @BeforeEach
    void setUp() {
        skill = new Fumblerooski();
        skill.postConstruct();
    }

    @Test
    void name_is_Fumblerooski() {
        assertEquals("Fumblerooski", skill.getName());
    }

    @Test
    void category_is_devious() {
        assertEquals(SkillCategory.DEVIOUS, skill.getCategory());
    }

    @Test
    void has_can_drop_ball_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canDropBall));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = Fumblerooski.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
