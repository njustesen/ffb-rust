/// 1:1 translation of com.fumbbl.ffb.server.commandline.InifileParamFilter.
use super::inifile_param_filter_result::InifileParamFilterResult;

pub struct InifileParamFilter;

impl InifileParamFilter {
    pub const DEFAULT_VALUE: &'static str = "server.ini";
    pub const INIFILE_PARAM: &'static str = "-inifile";
    pub const OVERRIDE_PARAM: &'static str = "-override";

    pub fn new() -> Self {
        Self
    }

    /// Strips `-inifile <name>` and `-override <file>` from args.
    /// Returns remaining args + extracted values; defaults inifile to "server.ini".
    pub fn filter_for_inifile(&self, args: &[String]) -> InifileParamFilterResult {
        let mut ini_file_name = Self::DEFAULT_VALUE.to_string();
        let mut override_file_name: Option<String> = None;
        let mut filtered_args: Vec<String> = Vec::new();

        let mut i = 0;
        while i < args.len() {
            if args[i] == Self::INIFILE_PARAM && i + 1 < args.len() {
                ini_file_name = args[i + 1].clone();
                i += 2;
            } else if args[i] == Self::OVERRIDE_PARAM && i + 1 < args.len() {
                override_file_name = Some(args[i + 1].clone());
                i += 2;
            } else {
                filtered_args.push(args[i].clone());
                i += 1;
            }
        }

        InifileParamFilterResult::new(ini_file_name, override_file_name, filtered_args)
    }
}

impl Default for InifileParamFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = InifileParamFilter::new();
    }

    #[test]
    fn default_inifile() {
        let filter = InifileParamFilter::new();
        let result = filter.filter_for_inifile(&[]);
        assert_eq!(result.get_ini_file_name(), "server.ini");
        assert!(result.get_override_file_name().is_none());
        assert_eq!(result.get_filtered_args().len(), 0);
    }

    #[test]
    fn extracts_inifile_param() {
        let filter = InifileParamFilter::new();
        let args: Vec<String> = vec!["-inifile".to_string(), "custom.ini".to_string(), "other".to_string()];
        let result = filter.filter_for_inifile(&args);
        assert_eq!(result.get_ini_file_name(), "custom.ini");
        assert_eq!(result.get_filtered_args(), &["other".to_string()]);
    }

    #[test]
    fn extracts_override_param() {
        let filter = InifileParamFilter::new();
        let args: Vec<String> = vec!["-override".to_string(), "override.ini".to_string()];
        let result = filter.filter_for_inifile(&args);
        assert_eq!(result.get_override_file_name(), Some("override.ini"));
        assert_eq!(result.get_ini_file_name(), "server.ini");
    }
}
