package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.Loner;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class LonerSkillTest {

    private Loner skill;

    @BeforeEach
    void setUp() {
        skill = new Loner();
        skill.postConstruct();
    }

    @Test
    void name_is_Loner() {
        assertEquals("Loner", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_has_to_roll_to_use_team_reroll_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.hasToRollToUseTeamReroll));
    }

    @Test
    void class_name_is_Loner() {
        assertEquals("Loner", skill.getClass().getSimpleName());
    }
}
