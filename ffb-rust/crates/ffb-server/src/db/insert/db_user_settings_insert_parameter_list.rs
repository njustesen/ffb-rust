/// 1:1 translation of com.fumbbl.ffb.server.db.insert.DbUserSettingsInsertParameterList.
use super::db_user_settings_insert_parameter::DbUserSettingsInsertParameter;

pub struct DbUserSettingsInsertParameterList {
    parameters: Vec<DbUserSettingsInsertParameter>,
}

impl DbUserSettingsInsertParameterList {
    pub fn new() -> Self {
        Self { parameters: Vec::new() }
    }

    pub fn add_parameter(&mut self, parameter: DbUserSettingsInsertParameter) {
        self.parameters.push(parameter);
    }

    pub fn add_parameter_values(
        &mut self,
        coach: String,
        setting_name: String,
        setting_value: String,
    ) {
        self.parameters.push(DbUserSettingsInsertParameter::new(
            coach,
            setting_name,
            setting_value,
        ));
    }

    pub fn get_parameters(&self) -> &[DbUserSettingsInsertParameter] {
        &self.parameters
    }
}

impl Default for DbUserSettingsInsertParameterList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let list = DbUserSettingsInsertParameterList::new();
        assert_eq!(list.get_parameters().len(), 0);
    }

    #[test]
    fn add_parameter_values() {
        let mut list = DbUserSettingsInsertParameterList::new();
        list.add_parameter_values("c".to_string(), "n".to_string(), "v".to_string());
        assert_eq!(list.get_parameters().len(), 1);
    }
}
