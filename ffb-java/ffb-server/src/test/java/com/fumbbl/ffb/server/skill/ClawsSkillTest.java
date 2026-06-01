package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.Claws;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ClawsSkillTest {

    private Claws skill;

    @BeforeEach
    void setUp() {
        skill = new Claws();
        skill.postConstruct();
    }

    @Test
    void name_is_Claws() {
        assertEquals("Claws", skill.getName());
    }

    @Test
    void category_is_mutation() {
        assertEquals(SkillCategory.MUTATION, skill.getCategory());
    }

    @Test
    void has_reduces_armour_to_fixed_value_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.reducesArmourToFixedValue));
    }

    @Test
    void class_name_is_Claws() {
        assertEquals("Claws", skill.getClass().getSimpleName());
    }
}
