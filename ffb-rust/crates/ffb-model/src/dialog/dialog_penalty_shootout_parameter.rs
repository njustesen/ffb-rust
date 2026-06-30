use serde::{Deserialize, Serialize};
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;
use crate::model::SoundId;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogPenaltyShootoutParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogPenaltyShootoutParameter {
    pub home_rolls: Vec<i32>,
    pub away_rolls: Vec<i32>,
    pub home_won: Vec<bool>,
    pub descriptions: Vec<String>,
    pub home_score: i32,
    pub away_score: i32,
    pub home_team_wins: bool,
    pub winning_sound: Option<SoundId>,
    pub losing_sound: Option<SoundId>,
}

impl DialogPenaltyShootoutParameter {
    pub fn add_shootout(&mut self, home: i32, away: i32, home_win: bool, round: String) {
        self.home_rolls.push(home);
        self.away_rolls.push(away);
        self.home_won.push(home_win);
        self.descriptions.push(round);
    }

    pub fn home_team_wins(&self) -> bool { self.home_team_wins }
    pub fn get_home_rolls(&self) -> &[i32] { &self.home_rolls }
    pub fn get_away_rolls(&self) -> &[i32] { &self.away_rolls }
    pub fn get_home_won(&self) -> &[bool] { &self.home_won }
    pub fn get_descriptions(&self) -> &[String] { &self.descriptions }
    pub fn get_home_score(&self) -> i32 { self.home_score }
    pub fn get_away_score(&self) -> i32 { self.away_score }
    pub fn get_winning_sound(&self) -> Option<SoundId> { self.winning_sound }
    pub fn get_losing_sound(&self) -> Option<SoundId> { self.losing_sound }
}

impl IDialogParameter for DialogPenaltyShootoutParameter {
    fn get_id(&self) -> DialogId { DialogId::PENALTY_SHOOTOUT }
    fn transform(&self) -> Box<dyn IDialogParameter> {
        Box::new(DialogPenaltyShootoutParameter {
            home_rolls: self.away_rolls.clone(),
            away_rolls: self.home_rolls.clone(),
            home_won: self.home_won.iter().map(|w| !w).collect(),
            descriptions: self.descriptions.clone(),
            home_score: self.away_score,
            away_score: self.home_score,
            home_team_wins: !self.home_team_wins,
            winning_sound: self.winning_sound,
            losing_sound: self.losing_sound,
        })
    }
}
