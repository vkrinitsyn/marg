mod feature;
pub mod token;
pub mod key;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use crate::feature::{featured, SupportedDb};
use crate::key::KeyFile;

const CMD_FILE: &str = "file";
const CMD_DB: &str = "db";
const CMD_TBL: &str = "config";
const CMD_TOKEN: &str = "token";
const CMD_TTL: &str = "ttl";
const CMD_KEY: &str = "key";
const CMD_PASS: &str = "PASSPHRASE";


/// App startup args:
/// - db connection url: usually the first arg
///   - or prefix with: '--db '
///     (optional, default postgres:// )
///
/// - config table name in format: schema.table, usually the second arg.
///   - or prefix with: '--config '
///     (optional, default public.{the_appname})
///
/// - token (db pwd) script name usually the third arg (required feature 'token')
///   - or prefix with '--token '
///
/// - token live in minutes, usually the forth arg (required feature 'token')
///   - or prefix with '--ttl '
///
/// - Private Key text file name to use with RSA OR AES encryption (required feature 'rsa')
///   - or prefix with '--key '
///
/// Alternative configuration:
/// - file name, usually the first arg
///   - or prefix with '--file '
///
/// File format:
///  - db: OR db=
///  - config: OR config=
///  - token: OR token=
///  - ttl: OR ttl=
///  - pk: OR pk=
///
/// params passed in cmd line override params loaded from file & env.
///
/// env:
/// PGPASSWORD, in case of postgres db url, use to connect to the DB
/// PASSPHRASE, in case of RSA private key required a passphrase
///
#[derive(Debug, Clone)]
pub struct ArgConfig {
    pub db_url: String,

    /// Format: schema.talbe
    pub table: String,

    pub cfg: HashMap<String, String>,

    pub token: token::Token,

    pub pk: Option<KeyFile>,
}


impl ArgConfig {
    #[cfg(not(feature="key"))]
    pub fn from_args() -> Result<ArgConfig, String> {
        ArgConfig::from_args_impl(None)
    }

    #[cfg(feature="key")]
    pub fn from_args(file_def: Option<&str>) -> Result<ArgConfig, String> {
        ArgConfig::from_args_impl(file_def)
    }

    fn from_args_impl(file_def: Option<&str>) -> Result<ArgConfig, String> {
        let user = match std::env::var_os("USER") {
            Some(a) => a.to_str().unwrap_or("postgres").to_string(),
            _ => "postgres".to_string(),
        };
        let f = featured();
        let pwd = if !f.env_pwd().is_empty() {
            match std::env::var_os(f.env_pwd().as_str()).map(|v| v) {
                Some(a) => a.to_str().map(|v| v.to_string()),
                _ => None,
            }
        } else {
            None
        };
        let input: Vec<String> = std::env::args_os().map(|e| e.to_string_lossy().to_string()).collect();

        ArgConfig::new(input, f, user, pwd, file_def)
    }

    /// first arg is an app itself
    fn new(input: Vec<String>, feature: SupportedDb, user: String, pwd: Option<String>, file_def: Option<&str>) -> Result<Self, String> {
        let mut cfg = HashMap::new();
        let mut db: Option<String> = None;
        let mut tbl: Option<String>  = None;
        let mut token: Option<String>  = None;
        let mut ttl: Option<String>  = None;
        let mut pk: Option<String>  = None;
        let mut ignore_next = true;
        for i in 1..input.len() {
            if ignore_next {continue}
            if input[i].starts_with("--") {
                if i < input.len() - 1 {
                    let v = &input[i].as_str()[2..];
                    if v == CMD_FILE {
                        let _ = load(v, &mut cfg)?;
                        ignore_next = true;
                    } else if v == CMD_DB {
                        db = Some(input[i + 1].to_string());
                        ignore_next = true;
                    } else if v == CMD_TBL {
                        tbl = Some(input[i + 1].to_string());
                        ignore_next = true;
                    } else if v == CMD_TOKEN {
                        token = Some(input[i + 1].to_string());
                        ignore_next = true;
                    } else if v == CMD_TTL {
                        ttl = Some(input[i + 1].to_string());
                        ignore_next = true;
                    } else if v == CMD_KEY {
                        let file = input[i + 1].to_string();
                        if key::is_key_file(&file) {
                            pk = Some(file);
                            ignore_next = true;
                        }
                    }
                }
            } else {
            }
        }
        // first was a check by tag names, then try to guess
        for i in &input {
            if i.starts_with("--") {
                continue
            }
            if db.is_none() && feature.is_valid_url(i) {
                db = Some(i.to_string());
                continue
            }
            if tbl.is_none() && is_sound_schema_table(i) {
                tbl = Some(i.to_string());
                continue
            }
            if token.is_none() && i.len() > 1 {
                token = Some(i.to_string());
                continue
            }
            if ttl.is_none() && i.parse::<u16>().is_ok() {
                ttl = Some(i.to_string());

            }
        }

        Ok(ArgConfig {
            db_url: link_db_user(db.unwrap_or(get_env_or_cfg(CMD_DB, &cfg, feature.default_url(&user).as_str())), user),
            table: tbl.unwrap_or(get_env_or_cfg(CMD_TBL, &cfg, get_exec_name("public.", input[0].as_str()).as_str())),
            token: token::Token::new(
                token.unwrap_or(get_env_or_cfg(CMD_TOKEN, &cfg, "")),
                ttl.unwrap_or(get_env_or_cfg(CMD_TTL, &cfg, "1")),
                pwd
            )?,
            pk: key::KeyFile::new(
                pk.unwrap_or(get_env_or_cfg(CMD_KEY, &cfg, file_def.as_ref().unwrap_or(&""))),
                std::env::var_os(CMD_PASS).map(|p| p.to_string_lossy().to_string()),
                file_def
            )?,
            cfg,
        })
    }

    /// append with password if $PWD present in 'db'
    pub fn db_url(&self) -> String {
        let url = self.db_url.clone();
        if let Some(i) = url.find(":$P") {
            if let Some(y) = url.find("@") {
                let pwd = url.as_str()[i+1..y].to_owned();
                return url.replace(
                    pwd.as_str(),
                    self.token.value.clone().unwrap_or("".into()).as_str()).to_string();
            }
        }
        url
    }
}

#[inline]
fn link_db_user(url: String, user: String) -> String {
    url.replace("$USER", user.as_str())
}

#[inline]
fn get_env_or_cfg(input: &str, cfg: &HashMap<String, String>, def: &str) -> String {
    match std::env::var_os(input).map(|v| v) {
        Some(a) => a.to_str().map(|v| v.to_string()).unwrap_or(def.to_string()),
        None => cfg.get(input).unwrap_or(&def.to_string()).into()
    }
}

/// Safe taking value
#[inline]
fn get_exec_name(schema: &str, input: &str) -> String {
    if input.is_empty() {
        return "".to_string();
    }
    let e = Path::new(input);
    let name = e.file_name().map(|f|f.to_str().unwrap_or("")).unwrap_or("").to_string();
    #[cfg(windows)]
    let name = name.replace(".exe", "");
    let name = if let Some(i) = name.rfind(std::path::MAIN_SEPARATOR_STR) {
        name[i+1..].to_string()
    } else {
        name
    };
    format!("{}{}", schema, name)
}

#[inline]
fn is_sound_schema_table(input: &str) -> bool {
    let y = input.contains(".");
    #[cfg(windows)]
    let y = y && !input.ends_with(".exe");
    y
}

#[inline]
fn load(file: &str, cfg: &mut HashMap<String, String>) -> Result<(), String> {
    let f = File::open(file).map_err(|e| e.to_string())?;
    let reader = BufReader::new(f);
    for line in reader.lines() {
        if let Ok(l) = line {
            if let Some(i) = l.chars().position(|c| c == '=' || c == ':' || c == '#' || c == ';' || c == '/' || c == '[') {
                if l.as_bytes()[i] != b'#' && l.as_bytes()[i] != b';' && l.as_bytes()[i] != b'/' && l.as_bytes()[i] != b'[' {
                    let key = l[..i].trim().to_lowercase();
                    let value = l[i + 1..].trim().to_string();
                    cfg.insert(key, value);
                }
            }
        }
    }
    Ok(())
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    #[cfg(unix)]
    fn test_file_name() {
        assert_eq!(get_exec_name("","").as_str(), "");
        assert_eq!(get_exec_name("","target/debug/marg").as_str(), "marg");
        assert_eq!(get_exec_name("","marg").as_str(), "marg");
    }

    #[test]
    #[cfg(windows)]
    fn test_file_name() {
        assert_eq!(get_exec_name("","").as_str(), "");
        assert_eq!(get_exec_name("","target\\debug\\marg.exe").as_str(), "marg");
        assert_eq!(get_exec_name("","target\\\\debug\\\\marg.exe").as_str(), "marg");
    }

    #[test]
    fn config_args_file1_test() {
        //
        let cfg = ArgConfig::new(
            vec!["".to_string()],
                 SupportedDb::Postgres, "".to_string(), None, None).unwrap();
        assert_eq!(0, cfg.cfg.len());

    }

    #[test]
    fn config_args_file2_test() {
        let url =  "postgresql://user:pwd@host/db".to_string();
        let cfg = ArgConfig::new(
            vec![url.clone()],
                 SupportedDb::Postgres, "vk".to_string(), None, None).unwrap();
        assert_eq!(url, cfg.db_url());
    }

    #[test]
    fn config_args_file3_test() {
        let url =  "postgresql://user:pwd@host/db".to_string();
        let t = "public.table".to_string();
        let cfg = ArgConfig::new(
            vec![url.clone(), t.clone()],
                 SupportedDb::Postgres, "".to_string(), None, None).unwrap();
        assert_eq!(url, cfg.db_url());
        assert_eq!(t, cfg.table);
    }

    #[test]
    fn config_args_user_test() {
        let user = match std::env::var_os("USER") {
            Some(a) => a.to_str().unwrap_or("postgres").to_string(),
            _ => "postgres".to_string(),
        };

        assert_eq!(format!("postgresql://{}:pwd@host/db", user), link_db_user("postgresql://$USER:pwd@host/db".to_string(), user.clone()));
        assert_eq!(format!("postgresql://{}:$PWD@host/db", user), link_db_user("postgresql://$USER:$PWD@host/db".to_string(), user.clone()));
        assert_eq!(format!("postgresql://{}@host/db", user), link_db_user("postgresql://$USER@host/db".to_string(), user.clone()));
        assert_eq!(format!("postgresql://{}@host/db", ""), link_db_user("postgresql://@host/db".to_string(), user.clone()));

    }


}
