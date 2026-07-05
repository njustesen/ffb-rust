/// 1:1 translation of `com.fumbbl.ffb.server.ServerMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerMode {
    Standalone,
    Fumbbl,
    StandaloneInitDb,
    FumbblInitDb,
}

impl ServerMode {
    pub fn get_name(&self) -> &'static str {
        match self {
            ServerMode::Standalone => "standalone",
            ServerMode::Fumbbl => "fumbbl",
            ServerMode::StandaloneInitDb => "standaloneInitDb",
            ServerMode::FumbblInitDb => "fumbblInitDb",
        }
    }

    pub fn is_standalone(&self) -> bool {
        matches!(self, ServerMode::Standalone | ServerMode::StandaloneInitDb)
    }

    pub fn is_init_db(&self) -> bool {
        matches!(self, ServerMode::StandaloneInitDb | ServerMode::FumbblInitDb)
    }

    /// 1:1 translation of `ServerMode.fromArguments(String[])`.
    /// Returns `None` when no arguments match (Java returns `null`).
    pub fn from_arguments(arguments: &[&str]) -> Option<ServerMode> {
        if arguments.is_empty() {
            return None;
        }
        if arguments[0].eq_ignore_ascii_case("fumbbl") {
            if arguments.len() < 2 {
                return Some(ServerMode::Fumbbl);
            }
            if arguments[1].eq_ignore_ascii_case("initdb") {
                return Some(ServerMode::FumbblInitDb);
            }
        }
        if arguments[0].eq_ignore_ascii_case("standalone") {
            if arguments.len() < 2 {
                return Some(ServerMode::Standalone);
            }
            if arguments[1].eq_ignore_ascii_case("initdb") {
                return Some(ServerMode::StandaloneInitDb);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_name_matches_java_strings() {
        assert_eq!(ServerMode::Standalone.get_name(), "standalone");
        assert_eq!(ServerMode::Fumbbl.get_name(), "fumbbl");
        assert_eq!(ServerMode::StandaloneInitDb.get_name(), "standaloneInitDb");
        assert_eq!(ServerMode::FumbblInitDb.get_name(), "fumbblInitDb");
    }

    #[test]
    fn is_standalone_only_for_standalone_variants() {
        assert!(ServerMode::Standalone.is_standalone());
        assert!(ServerMode::StandaloneInitDb.is_standalone());
        assert!(!ServerMode::Fumbbl.is_standalone());
        assert!(!ServerMode::FumbblInitDb.is_standalone());
    }

    #[test]
    fn is_init_db_for_init_db_variants() {
        assert!(ServerMode::StandaloneInitDb.is_init_db());
        assert!(ServerMode::FumbblInitDb.is_init_db());
        assert!(!ServerMode::Standalone.is_init_db());
        assert!(!ServerMode::Fumbbl.is_init_db());
    }

    #[test]
    fn from_arguments_fumbbl_no_args() {
        assert_eq!(ServerMode::from_arguments(&["fumbbl"]), Some(ServerMode::Fumbbl));
    }

    #[test]
    fn from_arguments_fumbbl_initdb() {
        assert_eq!(ServerMode::from_arguments(&["fumbbl", "initdb"]), Some(ServerMode::FumbblInitDb));
    }

    #[test]
    fn from_arguments_standalone_initdb() {
        assert_eq!(ServerMode::from_arguments(&["standalone", "initdb"]), Some(ServerMode::StandaloneInitDb));
    }

    #[test]
    fn from_arguments_empty_returns_none() {
        assert_eq!(ServerMode::from_arguments(&[]), None);
    }

    #[test]
    fn from_arguments_unknown_returns_none() {
        assert_eq!(ServerMode::from_arguments(&["unknown"]), None);
    }

    #[test]
    fn from_arguments_case_insensitive() {
        assert_eq!(ServerMode::from_arguments(&["FUMBBL"]), Some(ServerMode::Fumbbl));
        assert_eq!(ServerMode::from_arguments(&["Standalone", "InitDb"]), Some(ServerMode::StandaloneInitDb));
    }
}
