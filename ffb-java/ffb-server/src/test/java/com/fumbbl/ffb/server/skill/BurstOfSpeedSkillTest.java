package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.special.BurstOfSpeed;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BurstOfSpeedSkillTest {

    private BurstOfSpeed skill;

    @BeforeEach
    void setUp() {
        skill = new BurstOfSpeed();
        skill.postConstruct();
    }

    @Test
    void name_is_Burst_of_Speed() {
        assertEquals("Burst of Speed", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_make_an_extra_gfi_once_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canMakeAnExtraGfiOnce));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = BurstOfSpeed.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
