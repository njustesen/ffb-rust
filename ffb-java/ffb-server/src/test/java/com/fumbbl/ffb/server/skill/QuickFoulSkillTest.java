package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.QuickFoul;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class QuickFoulSkillTest {

    private QuickFoul skill;

    @BeforeEach
    void setUp() {
        skill = new QuickFoul();
        skill.postConstruct();
    }

    @Test
    void name_is_Quick_Foul() {
        assertEquals("Quick Foul", skill.getName());
    }

    @Test
    void category_is_devious() {
        assertEquals(SkillCategory.DEVIOUS, skill.getCategory());
    }

    @Test
    void has_can_move_after_foul_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canMoveAfterFoul));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = QuickFoul.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
