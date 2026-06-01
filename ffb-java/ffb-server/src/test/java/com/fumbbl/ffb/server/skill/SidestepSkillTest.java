package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.SideStep;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SideStepSkillTest {

    private SideStep skill;

    @BeforeEach
    void setUp() {
        skill = new SideStep();
        skill.postConstruct();
    }

    @Test
    void name_is_Side_Step() {
        assertEquals("Side Step", skill.getName());
    }

    @Test
    void category_is_agility() {
        assertEquals(SkillCategory.AGILITY, skill.getCategory());
    }

    @Test
    void has_can_choose_own_pushed_back_square_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canChooseOwnPushedBackSquare));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = SideStep.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
