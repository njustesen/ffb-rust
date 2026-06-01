package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.Brawler;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class BrawlerSkillTest {

    private Brawler skill;

    @BeforeEach
    void setUp() {
        skill = new Brawler();
        skill.postConstruct();
    }

    @Test
    void name_is_Brawler() {
        assertEquals("Brawler", skill.getName());
    }

    @Test
    void category_is_strength() {
        assertEquals(SkillCategory.STRENGTH, skill.getCategory());
    }

    @Test
    void has_can_reroll_single_both_down_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canRerollSingleBothDown));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = Brawler.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
