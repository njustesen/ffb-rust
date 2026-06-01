package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.special.TwoForOne;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class TwoForOneSkillTest {

    private TwoForOne skill;

    @BeforeEach
    void setUp() {
        skill = new TwoForOne();
        skill.postConstruct();
    }

    @Test
    void name_is_Two_for_One() {
        assertEquals("Two for One", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_reduces_loner_roll_if_partner_is_hurt_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.reducesLonerRollIfPartnerIsHurt));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = TwoForOne.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
