use std::fs::File;
/*
Set of features
*/
#[cfg(feature="postgres")]
const FEATURE_POSTGRES: bool = true;

#[cfg(not(feature="postgres"))]
const FEATURE_POSTGRES: bool = false;

#[cfg(feature="sqlite")]
const FEATURE_SQLITE: bool = true;

#[cfg(not(feature="sqlite"))]
const FEATURE_SQLITE: bool = false;


pub enum SupportedDb {
    /// no db pattern configured
    Custom,
    /// postgres:
    Postgres,
    Sqlite
}

pub(crate) fn featured() -> SupportedDb {
    if FEATURE_POSTGRES {
        return SupportedDb::Postgres;
    }
    if FEATURE_SQLITE {
        return SupportedDb::Sqlite;
    }
    SupportedDb::Custom
}

impl SupportedDb {
    pub(crate) fn is_valid_url(&self, url: &String) -> bool {
        match &self {
            // if no feature set any first param is a valid connection url
            SupportedDb::Custom => true,
            SupportedDb::Postgres => url.starts_with("postgresql://"),
            SupportedDb::Sqlite =>
                url.contains("Data Source") || File::open(url).is_ok()
        }
    }

    pub(crate) fn default_url(&self, user: &String) -> String {
        match &self {
            SupportedDb::Postgres => {
                let url = SupportedDb::DEFAULT_PG_URL.to_string();
                if user.len() > 0 && url.contains("$USER") {
                    url.replace("$USER", user.as_str())
                } else {
                    url
                }
            },
            SupportedDb::Custom | SupportedDb::Sqlite => "".to_string()
        }
    }

    pub const DEFAULT_PG_URL: &'static str = "postgresql://$USER@postgres?host=/var/run/postgresql";

    /// append enum dependent values in case of a new capabilities
    pub(crate) fn env_pwd(&self) -> String {
        "PGPASSWORD".into()
    }
}