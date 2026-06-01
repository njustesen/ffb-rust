package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.NurglesRot;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class NurglesRotSkillTest {

    private NurglesRot skill;

    @BeforeEach
    void setUp() {
        skill = new NurglesRot();
        skill.postConstruct();
    }

    @Test
    void name_is_Nurgles_Rot() {
        assertEquals("Nurgle's Rot", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_allows_raising_lineman_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.allowsRaisingLineman));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = NurglesRot.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
