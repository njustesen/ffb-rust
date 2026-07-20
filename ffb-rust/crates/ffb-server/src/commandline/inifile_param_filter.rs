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

        // Java iterates with an Iterator: the flag itself is always consumed
        // (and dropped), the value only if another arg follows.
        let mut i = 0;
        while i < args.len() {
            if args[i] == Self::INIFILE_PARAM {
                if i + 1 < args.len() {
                    ini_file_name = args[i + 1].clone();
                    i += 1;
                }
            } else if args[i] == Self::OVERRIDE_PARAM {
                if i + 1 < args.len() {
                    override_file_name = Some(args[i + 1].clone());
                    i += 1;
                }
            } else {
                filtered_args.push(args[i].clone());
            }
            i += 1;
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
    // Mirrors ffb-java ffb-server InifileParamFilterTest (Java: InifileParamFilterTest.java).
    // Java-only: `filterForInifileReturnsDefaultForNullArray` — Rust slices cannot be null.
    use super::*;

    const OTHER_PARAM_1: &str = "otherParam1";
    const OTHER_PARAM_2: &str = "otherParam2";
    const INIFILE_PARAM: &str = "-inifile";
    const INIFILE_VALUE: &str = "inifile_value";
    const OVERRIDE_PARAM: &str = "-override";
    const OVERRIDE_VALUE: &str = "override_value";
    const DEFAULT_INIFUILE_VALUE: &str = "server.ini";

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    /// Java: `filterForInifileFiltersExistingParam`.
    #[test]
    fn filter_for_inifile_filters_existing_param() {
        let filter = InifileParamFilter::new();
        let input = args(&[OTHER_PARAM_1, INIFILE_PARAM, INIFILE_VALUE, OTHER_PARAM_2]);
        let expected = args(&[OTHER_PARAM_1, OTHER_PARAM_2]);
        let result = filter.filter_for_inifile(&input);
        assert_eq!(result.get_filtered_args(), expected.as_slice(), "Inifile param and value were not filtered correctly");
        assert_eq!(result.get_ini_file_name(), INIFILE_VALUE, "Inifile value has not been extracted correctly");
        assert!(result.get_override_file_name().is_none(), "No override was specified");
    }

    /// Java: `filterForInifileFiltersExistingParams`.
    #[test]
    fn filter_for_inifile_filters_existing_params() {
        let filter = InifileParamFilter::new();
        let input = args(&[OTHER_PARAM_1, INIFILE_PARAM, INIFILE_VALUE, OTHER_PARAM_2, OVERRIDE_PARAM, OVERRIDE_VALUE]);
        let expected = args(&[OTHER_PARAM_1, OTHER_PARAM_2]);
        let result = filter.filter_for_inifile(&input);
        assert_eq!(result.get_filtered_args(), expected.as_slice(), "Inifile param and value were not filtered correctly");
        assert_eq!(result.get_ini_file_name(), INIFILE_VALUE, "Inifile value has not been extracted correctly");
        assert_eq!(result.get_override_file_name(), Some(OVERRIDE_VALUE), "Override value has not been extracted correctly");
    }

    /// Java: `filterForInifileReturnsDefaultForMissingParam`.
    #[test]
    fn filter_for_inifile_returns_default_for_missing_param() {
        let filter = InifileParamFilter::new();
        let input = args(&[OTHER_PARAM_1, OTHER_PARAM_2]);
        let expected = args(&[OTHER_PARAM_1, OTHER_PARAM_2]);
        let result = filter.filter_for_inifile(&input);
        assert_eq!(result.get_filtered_args(), expected.as_slice(), "Other params must be retained as passed in.");
        assert_eq!(result.get_ini_file_name(), DEFAULT_INIFUILE_VALUE, "Inifile value must be set to the default value");
    }

    /// Java: `filterForInifileReturnsDefaultForMissingValue`. A trailing
    /// `-inifile` with no value is consumed and dropped, like Java's iterator.
    #[test]
    fn filter_for_inifile_returns_default_for_missing_value() {
        let filter = InifileParamFilter::new();
        let input = args(&[OTHER_PARAM_1, INIFILE_PARAM]);
        let expected = args(&[OTHER_PARAM_1]);
        let result = filter.filter_for_inifile(&input);
        assert_eq!(result.get_filtered_args(), expected.as_slice(), "Inifile param was not filtered correctly");
        assert_eq!(result.get_ini_file_name(), DEFAULT_INIFUILE_VALUE, "Inifile value must be set to the default value");
    }

    /// Java: `filterForInifileReturnsDefaultForEmptyArray`.
    #[test]
    fn filter_for_inifile_returns_default_for_empty_array() {
        let filter = InifileParamFilter::new();
        let result = filter.filter_for_inifile(&[]);
        assert_eq!(result.get_filtered_args().len(), 0, "Empty input must result in empty output");
        assert_eq!(result.get_ini_file_name(), DEFAULT_INIFUILE_VALUE, "Inifile value must be set to the default value");
    }

    /// Java: `filterForInifileExtractsOverrideParam` (ported Rust extra).
    #[test]
    fn filter_for_inifile_extracts_override_param() {
        let filter = InifileParamFilter::new();
        let input = args(&[OVERRIDE_PARAM, OVERRIDE_VALUE]);
        let result = filter.filter_for_inifile(&input);
        assert_eq!(result.get_override_file_name(), Some(OVERRIDE_VALUE));
        assert_eq!(result.get_ini_file_name(), DEFAULT_INIFUILE_VALUE);
    }
}
