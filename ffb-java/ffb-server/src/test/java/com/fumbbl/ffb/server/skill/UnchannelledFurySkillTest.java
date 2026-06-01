package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.UnchannelledFury;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class UnchannelledFurySkillTest {

    private UnchannelledFury skill;

    @BeforeEach
    void setUp() {
        skill = new UnchannelledFury();
        skill.postConstruct();
    }

    @Test
    void name_is_Unchannelled_Fury() {
        assertEquals("Unchannelled Fury", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_enable_stand_up_and_end_blitz_action_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.enableStandUpAndEndBlitzAction));
    }

    @Test
    void class_name_is_UnchannelledFury() {
        assertEquals("UnchannelledFury", skill.getClass().getSimpleName());
    }
}
