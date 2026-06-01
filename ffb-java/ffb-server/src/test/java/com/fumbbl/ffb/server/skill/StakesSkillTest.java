package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.Stakes;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class StakesSkillTest {

    private Stakes skill;

    @BeforeEach
    void setUp() {
        skill = new Stakes();
        skill.postConstruct();
    }

    @Test
    void name_is_Stakes() {
        assertEquals("Stakes", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_provides_stab_block_alternative_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.providesStabBlockAlternative));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = Stakes.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
