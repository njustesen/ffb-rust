/// 1:1 translation of com.fumbbl.ffb.server.db.insert.DbTeamSetupsInsertParameter.
use mysql_async::{prelude::Queryable, Conn, Error as DbError, Value};

use super::db_team_setups_insert::SQL;

pub struct DbTeamSetupsInsertParameter {
    team_id: String,
    name: String,
    player_numbers: Vec<u8>,
    x_coordinates: Vec<u8>,
    y_coordinates: Vec<u8>,
    updated_rows: i32,
}

impl DbTeamSetupsInsertParameter {
    pub fn new(
        team_id: String,
        name: String,
        player_numbers: Vec<u8>,
        x_coordinates: Vec<u8>,
        y_coordinates: Vec<u8>,
    ) -> Self {
        Self {
            team_id,
            name,
            player_numbers,
            x_coordinates,
            y_coordinates,
            updated_rows: 0,
        }
    }

    pub fn get_team_id(&self) -> &str {
        &self.team_id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_player_numbers(&self) -> &[u8] {
        &self.player_numbers
    }

    pub fn get_x_coordinates(&self) -> &[u8] {
        &self.x_coordinates
    }

    pub fn get_y_coordinates(&self) -> &[u8] {
        &self.y_coordinates
    }

    pub fn get_updated_rows(&self) -> i32 {
        self.updated_rows
    }

    /// Executes the INSERT statement. Mirrors DbTeamSetupsInsert.fillDbStatement():
    /// team_id, name, then 11 × (player_nr, x, y), NULL-padding any slots beyond the
    /// supplied array length.
    pub async fn execute(&mut self, conn: &mut Conn) -> Result<u64, DbError> {
        let mut params: Vec<Value> = Vec::with_capacity(35);
        params.push(Value::from(self.team_id.as_str()));
        params.push(Value::from(self.name.as_str()));
        let n = self.player_numbers.len().min(11);
        for i in 0..11usize {
            if i < n {
                params.push(Value::from(self.player_numbers[i]));
                params.push(Value::from(self.x_coordinates[i]));
                params.push(Value::from(self.y_coordinates[i]));
            } else {
                params.push(Value::NULL);
                params.push(Value::NULL);
                params.push(Value::NULL);
            }
        }
        conn.exec_drop(SQL, params).await?;
        let rows = conn.affected_rows();
        self.updated_rows = rows as i32;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let p = DbTeamSetupsInsertParameter::new(
            "t1".to_string(),
            "setup1".to_string(),
            vec![1, 2],
            vec![3, 4],
            vec![5, 6],
        );
        assert_eq!(p.get_team_id(), "t1");
        assert_eq!(p.get_name(), "setup1");
        assert_eq!(p.get_player_numbers().len(), 2);
    }

    #[test]
    fn get_updated_rows_initial() {
        let p = DbTeamSetupsInsertParameter::new(
            "t1".to_string(),
            "setup1".to_string(),
            vec![],
            vec![],
            vec![],
        );
        assert_eq!(p.get_updated_rows(), 0);
    }
}
