package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2020.special.WisdomOfTheWhiteDwarf;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class WisdomOfTheWhiteDwarfSkillTest {

    private WisdomOfTheWhiteDwarf skill;

    @BeforeEach
    void setUp() {
        skill = new WisdomOfTheWhiteDwarf();
        skill.postConstruct();
    }

    @Test
    void name_is_Wisdom_of_the_White_Dwarf() {
        assertEquals("Wisdom of the White Dwarf", skill.getName());
    }

    @Test
    void category_is_trait() {
        assertEquals(SkillCategory.TRAIT, skill.getCategory());
    }

    @Test
    void has_can_grant_skills_to_team_mates_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canGrantSkillsToTeamMates));
    }

    @Test
    void is_bb2020_edition() {
        RulesCollection annotation = WisdomOfTheWhiteDwarf.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2020, annotation.value());
    }
}
