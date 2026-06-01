package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.BigHand;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BigHandSkillTest {

    private BigHand skill;

    @BeforeEach
    void setUp() {
        skill = new BigHand();
        skill.postConstruct();
    }

    @Test
    void name_is_Big_Hand() {
        assertEquals("Big Hand", skill.getName());
    }

    @Test
    void category_is_mutation() {
        assertEquals(SkillCategory.MUTATION, skill.getCategory());
    }

    @Test
    void has_ignore_tacklezones_when_picking_up_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.ignoreTacklezonesWhenPickingUp));
    }

    @Test
    void class_name_is_BigHand() {
        assertEquals("BigHand", skill.getClass().getSimpleName());
    }
}
