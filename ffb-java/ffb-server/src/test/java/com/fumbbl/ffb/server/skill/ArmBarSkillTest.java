package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.ArmBar;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class ArmBarSkillTest {

    private ArmBar skill;

    @BeforeEach
    void setUp() {
        skill = new ArmBar();
        skill.postConstruct();
    }

    @Test
    void name_is_Arm_Bar() {
        assertEquals("Arm Bar", skill.getName());
    }

    @Test
    void category_is_strength() {
        assertEquals(SkillCategory.STRENGTH, skill.getCategory());
    }

    @Test
    void has_affects_either_armour_or_injury_on_dodge_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.affectsEitherArmourOrInjuryOnDodge));
    }

    @Test
    void class_name_is_ArmBar() {
        assertEquals("ArmBar", skill.getClass().getSimpleName());
    }
}
