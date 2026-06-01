package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.mixed.AnimalSavagery;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class AnimalSavagerySkillTest {

    private AnimalSavagery skill;

    @BeforeEach
    void setUp() {
        skill = new AnimalSavagery();
        skill.postConstruct();
    }

    @Test
    void name_is_Animal_Savagery() {
        assertEquals("Animal Savagery", skill.getName());
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
    void class_name_is_AnimalSavagery() {
        assertEquals("AnimalSavagery", skill.getClass().getSimpleName());
    }
}
