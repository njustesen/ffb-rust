package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.EyeGouge;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class EyeGougeSkillTest {

    private EyeGouge skill;

    @BeforeEach
    void setUp() {
        skill = new EyeGouge();
        skill.postConstruct();
    }

    @Test
    void name_is_Eye_Gouge() {
        assertEquals("Eye Gouge", skill.getName());
    }

    @Test
    void category_is_devious() {
        assertEquals(SkillCategory.DEVIOUS, skill.getCategory());
    }

    @Test
    void has_can_remove_opponent_assists_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canRemoveOpponentAssists));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = EyeGouge.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
