/// 1:1 translation of com.fumbbl.ffb.server.commandline.InifileParamFilterResult.
pub struct InifileParamFilterResult {
    ini_file_name: String,
    override_file_name: Option<String>,
    filtered_args: Vec<String>,
}

impl InifileParamFilterResult {
    pub fn new(
        ini_file_name: String,
        override_file_name: Option<String>,
        filtered_args: Vec<String>,
    ) -> Self {
        Self { ini_file_name, override_file_name, filtered_args }
    }

    pub fn get_ini_file_name(&self) -> &str {
        &self.ini_file_name
    }

    pub fn get_override_file_name(&self) -> Option<&str> {
        self.override_file_name.as_deref()
    }

    pub fn get_filtered_args(&self) -> &[String] {
        &self.filtered_args
    }
}
