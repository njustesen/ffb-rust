package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.WeepingDagger;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class WeepingDaggerSkillTest {

    private WeepingDagger skill;

    @BeforeEach
    void setUp() {
        skill = new WeepingDagger();
        skill.postConstruct();
    }

    @Test
    void name_is_Weeping_Dagger() {
        assertEquals("Weeping Dagger", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_applies_poison_on_badly_hurt_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.appliesPoisonOnBadlyHurt));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = WeepingDagger.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
