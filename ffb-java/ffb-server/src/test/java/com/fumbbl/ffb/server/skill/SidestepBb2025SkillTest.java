package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2025.Sidestep;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

// Named SidestepBb2025SkillTest (not SidestepSkillTest) because that file name is
// already taken by the bb2016 SideStepSkillTest and the two would collide on
// case-insensitive filesystems.
class SidestepBb2025SkillTest {

    private Sidestep skill;

    @BeforeEach
    void setUp() {
        skill = new Sidestep();
        skill.postConstruct();
    }

    @Test
    void name_is_Sidestep() {
        assertEquals("Sidestep", skill.getName());
    }

    @Test
    void category_is_agility() {
        assertEquals(SkillCategory.AGILITY, skill.getCategory());
    }

    @Test
    void has_can_choose_own_pushed_back_square_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.canChooseOwnPushedBackSquare),
            "Sidestep must register canChooseOwnPushedBackSquare so its coach picks the push-back square");
    }

    @Test
    void does_not_have_forceFollowup_property() {
        assertFalse(skill.hasSkillProperty(NamedProperties.forceFollowup),
            "Sidestep does not force follow-up");
    }

    @Test
    void is_bb2025_edition() {
        RulesCollection annotation = Sidestep.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2025, annotation.value());
    }
}
