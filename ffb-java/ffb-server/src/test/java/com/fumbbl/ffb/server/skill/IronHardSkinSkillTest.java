package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.IronHardSkin;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class IronHardSkinSkillTest {

    private IronHardSkin skill;

    @BeforeEach
    void setUp() {
        skill = new IronHardSkin();
        skill.postConstruct();
    }

    @Test
    void name_is_Iron_Hard_Skin() {
        assertEquals("Iron Hard Skin", skill.getName());
    }

    @Test
    void category_is_mutation() {
        assertEquals(SkillCategory.MUTATION, skill.getCategory());
    }

    @Test
    void has_ignores_armour_modifiers_from_fouls_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.ignoresArmourModifiersFromFouls));
    }

    @Test
    void class_name_is_IronHardSkin() {
        assertEquals("IronHardSkin", skill.getClass().getSimpleName());
    }
}
