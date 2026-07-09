pub struct IServerProperty;

impl IServerProperty {
    pub const SERVER_PORT: &'static str = "server.port";
    pub const SERVER_BASE: &'static str = "server.base";
    pub const SERVER_BASE_DIR: &'static str = "server.base.dir";
    pub const SERVER_LOG_FILE: &'static str = "server.log.file";
    pub const SERVER_LOG_FOLDER: &'static str = "server.log.folder";
    pub const SERVER_LOG_LEVEL: &'static str = "server.log.level";
    pub const SERVER_LOG_FILE_SPLIT: &'static str = "server.log.file.split";
    pub const SERVER_SPECTATOR_COOLDOWN: &'static str = "server.spectator.cooldown";
    pub const SERVER_COMMAND_COMPRESSION: &'static str = "server.command.compression";
    pub const SERVER_TEST: &'static str = "server.test";
    pub const SERVER_REDEPLOY_EXIT_CODE: &'static str = "server.redeploy.exitCode";
    pub const SERVER_REDEPLOY_DEFAULT_BRANCH: &'static str = "server.redeploy.defaultBranch";
    pub const SERVER_REDEPLOY_FILE: &'static str = "server.redeploy.file";
    pub const FUMBBL_USER: &'static str = "fumbbl.user";
    pub const FUMBBL_PASSWORD: &'static str = "fumbbl.password";
    pub const FUMBBL_BASE: &'static str = "fumbbl.base";
    pub const FUMBBL_PORT: &'static str = "fumbbl.port";
    pub const ADMIN_SALT: &'static str = "admin.salt";
    pub const ADMIN_PASSWORD: &'static str = "admin.password";
    pub const BACKUP_DIR: &'static str = "backup.dir";
    pub const BACKUP_EXTENSION: &'static str = "backup.extension";
    pub const BACKUP_SALT: &'static str = "backup.salt";
    pub const BACKUP_S3_PROFILE: &'static str = "backup.s3.profile";
    pub const BACKUP_S3_REGION: &'static str = "backup.s3.region";
    pub const BACKUP_S3_BUCKET: &'static str = "backup.s3.bucket";
    pub const BACKUP_S3_BASE_PATH: &'static str = "backup.s3.basePath";
    pub const DB_DRIVER: &'static str = "db.driver";
    pub const DB_URL: &'static str = "db.url";
    pub const DB_USER: &'static str = "db.user";
    pub const DB_PASSWORD: &'static str = "db.password";
    pub const DB_TYPE: &'static str = "db.type";
    pub const TIMER_DB_KEEP_ALIVE: &'static str = "timer.dbKeepAlive";
    pub const TIMER_NETWORK_ENTROPY: &'static str = "timer.networkEntropy";
    pub const TIMER_SESSION_TIMEOUT_ENABLED: &'static str = "timer.sessionTimeoutEnabled";
    pub const TIMER_SESSION_TIMEOUT_SCHEDULE: &'static str = "timer.sessionTimeoutSchedule";
    pub const SESSION_TIMEOUT_VALUE: &'static str = "session.timeoutValue";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_url_property_key() {
        assert_eq!(IServerProperty::DB_URL, "db.url");
    }

    #[test]
    fn test_server_port_property_key() {
        assert_eq!(IServerProperty::SERVER_PORT, "server.port");
    }
}
