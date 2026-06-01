package com.fumbbl.ffb.server.skill;

import com.fumbbl.ffb.RulesCollection;
import com.fumbbl.ffb.SkillCategory;
import com.fumbbl.ffb.model.property.NamedProperties;
import com.fumbbl.ffb.skill.bb2016.SecretWeapon;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SecretWeaponSkillTest {

    private SecretWeapon skill;

    @BeforeEach
    void setUp() {
        skill = new SecretWeapon();
        skill.postConstruct();
    }

    @Test
    void name_is_Secret_Weapon() {
        assertEquals("Secret Weapon", skill.getName());
    }

    @Test
    void category_is_extraordinary() {
        assertEquals(SkillCategory.EXTRAORDINARY, skill.getCategory());
    }

    @Test
    void has_gets_sent_off_at_end_of_drive_property() {
        assertTrue(skill.hasSkillProperty(NamedProperties.getsSentOffAtEndOfDrive));
    }

    @Test
    void is_bb2016_edition() {
        RulesCollection annotation = SecretWeapon.class.getAnnotation(RulesCollection.class);
        assertNotNull(annotation);
        assertEquals(RulesCollection.Rules.BB2016, annotation.value());
    }
}
