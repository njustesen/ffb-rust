package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.RunningPass;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class RunningPassSkillTest {

    private RunningPass skill;

    @BeforeEach
    void setUp() {
        skill = new RunningPass();
        skill.postConstruct();
    }

    @Test
    void name_is_Running_Pass() {
        assertEquals("Running Pass", skill.getName());
    }

    @Test
    void category_is_passing() {
        assertEquals(SkillCategory.PASSING, skill.getCategory());
    }

    @Test
    void has_can_move_after_quick_pass_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canMoveAfterQuickPass));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = RunningPass.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
