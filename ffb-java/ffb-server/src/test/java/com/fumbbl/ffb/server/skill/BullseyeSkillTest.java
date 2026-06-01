package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.Bullseye;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BullseyeSkillTest {

    private Bullseye skill;

    @BeforeEach
    void setUp() {
        skill = new Bullseye();
        skill.postConstruct();
    }

    @Test
    void name_is_Bullseye() {
        assertEquals("Bullseye", skill.getName());
    }

    @Test
    void category_is_strength() {
        assertEquals(SkillCategory.STRENGTH, skill.getCategory());
    }

    @Test
    void has_can_skip_ttm_scatter_on_superb_throw_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canSkipTtmScatterOnSuperbThrow));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = Bullseye.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
