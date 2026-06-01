package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.skill.mixed.special.KeenPlayer;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class KeenPlayerSkillTest {

    private KeenPlayer skill;

    @BeforeEach
    void setUp() {
        skill = new KeenPlayer();
        skill.postConstruct();
    }

    @Test
    void name_is_KeenPlayer() {
        assertEquals("Keen Player", skill.getName());
    }

    @Test
    void category_is_TRAIT() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_two_edition_annotations() {
        RulesCollection[] annotations = KeenPlayer.class.getDeclaredAnnotationsByType(RulesCollection.class);
        assertEquals(2, annotations.length);
    }

    @Test
    void has_skill_properties_not_null() {
        assertNotNull(skill.getSkillProperties());
    }
}
