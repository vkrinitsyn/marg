mod feature;
pub mod token;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use crate::feature::{featured, SupportedDb};

const CMD_FILE: &str = "file";
const CMD_DB: &str = "db";
const CMD_TBL: &str = "config";
const CMD_TOKEN: &str = "token";
const CMD_TTL: &str = "ttl";

/// App startup args:
/// - db connection url: usually the first arg
///   - or prefix with: '--db '
///     (optional, default postgres:// )
///
/// - config table name in format: schema.table, usually the second arg.
///   - or prefix with: '--config '
///     (optional, default public.{the_appname})
///
/// - token script name usually the third arg (required feature 'token')
///   - or prefix with '--token '
///
/// - token live in minutes, usually the forth arg (required feature 'token')
///   - or prefix with '--ttl '
///
/// Alternative configuration:
/// - file name, usually the first arg
///   - or prefix with '--file '
///
/// File format:
///  - db: OR db=
///  - config: OR config=
///  - token
///  - ttl
///
/// params passed in cmd line override params loaded from file & env.
///
/// env:
/// PGPASSWORD, in case of postgres db url, use to connect to the DB
///
#[derive(Debug, Clone)]
pub struct ArgConfig {
    pub db_url: String,

    /// Format: schema.talbe
    pub table: String,

    /// если первый аргумент в виде файла config file
    pub cfg: HashMap<String, String>,

    pub token: token::Token
}


impl ArgConfig {
    pub fn from_args() -> Result<ArgConfig, String> {
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

        ArgConfig::new(input, f, user, pwd)
    }

    /// first arg is an app itself
    fn new(input: Vec<String>, feature: SupportedDb, user: String, pwd: Option<String>) -> Result<Self, String> {
        let mut cfg = HashMap::new();
        let mut db: Option<String> = None;
        let mut tbl: Option<String>  = None;
        let mut token: Option<String>  = None;
        let mut ttl: Option<String>  = None;
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
                    }
                }
            } else {
            }
        }
        // first was a check by tag names, then try to guess

        ignore_next = true;

        for i in &input {
            if i.starts_with("--") {
                ignore_next = true;
            }
            if ignore_next {continue}
            if db.is_none() && feature.is_valid_url(i) {
                db = Some(i.to_string());
                continue
            }
            if tbl.is_none() && i.find(".").is_some() {
                tbl = Some(i.to_string());
                continue
            }
            if token.is_none() && i.len() > 1 {
                tbl = Some(i.to_string());
                continue
            }
            if tbl.is_none() && i.parse::<u16>().is_ok() {
                tbl = Some(i.to_string());

            }
        }

        Ok(ArgConfig {
            db_url: db.unwrap_or(get_env_or_cfg(CMD_DB, &cfg, feature.default_url(&user).as_str())),
            table: tbl.unwrap_or(get_env_or_cfg(CMD_TBL, &cfg, get_exec_name(input[0].as_str()).as_str())),
            token: token::Token::new(
                token.unwrap_or(get_env_or_cfg(CMD_TOKEN, &cfg, "")),
                ttl.unwrap_or(get_env_or_cfg(CMD_TTL, &cfg, "1")),
                pwd
            ),
            cfg,
        })
    }
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
fn get_exec_name(input: &str) -> String {
    let e = Path::new(input);
    if e.exists() {
        let f = e.file_name().map(|f|f.to_str().unwrap_or("")).unwrap_or("").to_string();
        f.replace(".exe", "")
    } else {
        "".into()
    }
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
        assert_eq!(get_exec_name("").as_str(), "");
        assert_eq!(get_exec_name("target/debug/marg").as_str(), "marg");
        assert_eq!(get_exec_name("marg").as_str(), "marg");
    }

    #[test]
    #[cfg(windows)]
    fn test_file_name() {
        assert_eq!(get_exec_name("").as_str(), "");
        assert_eq!(get_exec_name("target\\debug\\marg.exe").as_str(), "marg");
        assert_eq!(get_exec_name("target\\\\debug\\\\marg.exe").as_str(), "marg");
    }

    /*
    #[inline]
    fn uuid() -> String {
        let x = Uuid::new_v4();
        x.hyphenated().to_string()[..8].into()
    }

    #[test]
    fn config_args_file1_test() {
        let f = format!("/tmp/yt-{}.cfg", uuid());
        fs::write(&f, "db=p\nconfig=c").ok();
        let cfg = ["test".to_string(), f.clone()].to_vec();
        let cfg = ArgConfig::new(cfg, , ).unwrap();
        assert_eq!(cfg.db_url.as_str(), "p");
        assert_eq!(cfg.table.as_str(), "public.c");
        fs::remove_file(&f).ok();
    }

    #[test]
    fn config_args_file2_test() {
        let f = format!("/tmp/yt-{}.cfg", uuid());
        fs::write(&f, "db = p\nconfig : c").ok();
        let cfg = ["test".to_string(), f.clone()].to_vec();
        let cfg = ArgConfig::new(cfg, , ).unwrap();
        assert_eq!(cfg.db_url.as_str(), "p");
        assert_eq!(cfg.table.as_str(), "public.c");
        fs::remove_file(&f).ok();
    }

    #[test]
    fn config_args_1_test() {
        let cfg = ["test".to_string(), "postgresql://db".to_string()].to_vec();
        let cfg = ArgConfig::new(cfg, , ).unwrap();
        assert_eq!(cfg.db_url.as_str(), "postgresql://db");
    }

    #[test]
    fn config_args_2_test() {
        let cfg = ["test".to_string(), "postgresql://db".to_string(), "c".to_string()].to_vec();
        let cfg = ArgConfig::new(cfg, , ).unwrap();
        assert_eq!(cfg.db_url.as_str(), "postgresql://db");
        assert_eq!(cfg.table.as_str(), "public.c");
    }

    #[test]
    fn config_args_3_test() {
        let cfg = ["test".to_string(), "db".to_string(), "c".to_string(), "c".to_string()].to_vec();
        match ArgConfig::new(cfg, , ) {
            Ok(_) => {
                assert!(false);
            }
            Err(_) => {
                assert!(true)
            }
        }
    }

    // #[test]
    fn config_args_4_test() {
        match ArgConfig::new(Vec::new(), , ) {
            Ok(_) => {
                assert!(false);
            }
            Err(_) => {
                assert!(true)
            }
        }
    }


    #[test]
    fn config_args_parsing_test() {
        assert_eq!("".to_string(), ArgConfig::parse_table_name(&"".to_string(), &"".to_string()));
        assert_eq!(format!("s.{}", ytcc::DEFAULT_TABLE),
                   ArgConfig::parse_table_name(&"".to_string(), &"s.".to_string()));
        assert_eq!(format!("{}.t", ytcc::DEFAULT_SCHEMA),
                   ArgConfig::parse_table_name(&"".to_string(), &".t".to_string()));
        assert_eq!(format!("{}.t", ytcc::DEFAULT_SCHEMA),
                   ArgConfig::parse_table_name(&"".to_string(), &"t".to_string()));
    }
*/
}
