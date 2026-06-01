package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.AlwaysHungry;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class AlwaysHungrySkillTest {

    private AlwaysHungry skill;

    @BeforeEach
    void setUp() {
        skill = new AlwaysHungry();
        skill.postConstruct();
    }

    @Test
    void name_is_Always_Hungry() {
        assertEquals("Always Hungry", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_might_eat_player_to_throw_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.mightEatPlayerToThrow));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = AlwaysHungry.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
