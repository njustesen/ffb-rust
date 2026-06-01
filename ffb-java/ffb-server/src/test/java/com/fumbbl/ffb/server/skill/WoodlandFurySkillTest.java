package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.special.WoodlandFury;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class WoodlandFurySkillTest {

    private WoodlandFury skill;

    @BeforeEach
    void setUp() {
        skill = new WoodlandFury();
        skill.postConstruct();
    }

    @Test
    void name_is_Woodland_Fury() {
        assertEquals("Woodland Fury", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_reroll_single_block_die_when_would_be_knocked_down_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canRerollSingleBlockDieWhenWouldBeKnockedDown));
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = WoodlandFury.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
