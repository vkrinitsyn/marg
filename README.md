# marg

[![ci](https://github.com/vkrinitsyn/marg/actions/workflows/ci.yml/badge.svg)](https://github.com/vkrinitsyn/marg/actions/workflows/ci.yml)
[![coverage](https://raw.githubusercontent.com/vkrinitsyn/marg/badges/coverage.svg?sanitize=true)](https://github.com/vkrinitsyn/marg/actions/workflows/ci.yml)
[![audit](https://github.com/vkrinitsyn/marg/actions/workflows/audit.yml/badge.svg)](https://github.com/vkrinitsyn/marg/actions/workflows/audit.yml)
[![crates.io](https://img.shields.io/crates/v/marg.svg)](https://crates.io/crates/marg)
[![docs.rs](https://docs.rs/marg/badge.svg)](https://docs.rs/marg)
[![license](https://img.shields.io/crates/l/marg.svg)](LICENSE)

Meta config for apps from args
Use to connect to DB to pull other configs

## The idea

An app built on marg starts in three steps, and marg is the first:

1. **The minimum, to reach the database.** The command line - or a file, or the
   environment - carries only what connecting needs: the URL, the config table, the
   instance id, the secret. Nothing about what the app does.
2. **Everything else from the database.** Once connected, the app reads its settings from
   its config table. They change at run time, are shared by every instance that reads the
   same table, and are replicated with the data where the database is - a dynamic,
   flexible configuration instead of a static file per host.
3. **As simple as possible to start.** Every parameter is optional and guessed: a URL by
   its `postgres://` prefix, a table by its `schema.table` form, an id by its UUID form, in
   any order. With none given, the defaults - the local socket as `$USER`, the table named
   after the binary - are enough to start.

YT Cluster's `ytserv` and `ytctl` both start this way. A node's service runs `ytserv` with
two arguments - the socket URL and the config table:
`ytserv 'postgresql://postgres@/app?host=/var/run/postgresql&port=5432' public.yt_config` -
and reads everything else from that table; `ytctl` alone opens its dashboard with every
default ([how the apps start](https://docs.dbinvent.com/yt/guide/app-start)).

> [!NOTE]
> Also look at [Schema Guard](https://github.com/vkrinitsyn/schema_guard) , YAML based schema
> management for Postgres - both provided by [DBinvent](https://www.dbinvent.com), the infinite-scale
> Postgres solution.

support external app execution (az acc) for a password (token) acquisition

## ArgConfig

Parses app startup arguments into a structured config. 

### Priority order:
1. command-line args: 
   1. prefixed with --name
   2. positional or heuristic detection
2. environment variables
3. config file.

### Flags

Boolean flags that control runtime behaviour. Check `cfg.verbose.is_set()` etc. in your application.

| Flag | Aliases | Description                                                  |
|------|---------|--------------------------------------------------------------|
| `-v` | `--verbose`, `--debug` | Enable verbose output                                        |
| `-q` | `--quiet`, `--batch` | Suppress non-essential output or focus on non-interractive   |
| `-h` | `--help`, `--info` | Caller should print help                                     |
| `-V` | `--version` | Prints `<name> <version>` and exits (handled by `from_args`) |

Flags also accept explicit values: `--verbose=true` / `--verbose=false`.

### Arguments

Arguments can be passed positionally (auto-detected by format) or with explicit `--flag value` prefixes:

| Flag | Positional Or Detection | Description | Default |
|------|----------------------|-------------|---------|
| `--db` | URL prefix (e.g. `postgres://`) | Database connection URL | `postgres://` with current `$USER` |
| `--config` | `schema.table` format (contains `.`) | Config table name | `public.<appname>`* |
| `--uuid` | UUID-formatted string | Node/instance ID | Auto-generated (see `uuid_gen`) |
| `--token` | 3rd unmatched arg | Script name to acquire DB password | _(none)_ |
| `--ttl` | Numeric string | Token lifetime in minutes | `1` |
| `--key` | _(explicit only)_ | RSA/AES private key file path (feature `rsa`) | _(none)_ |
| `--secret` | _(explicit only)_ | AES cipher secret (prefer env `SECRET`) | _(none)_ |
| `--file` | _(explicit only)_ | Config file to load key=value pairs from | _(none)_ |

> *binary name truncated at the first `-` or `_`, e.g. `marg-server` → `public.marg`

### Environment Variables

- `PGPASSWORD` - PostgreSQL password (used to connect to DB)
- `PASSPHRASE` - Passphrase for RSA private key
- `SECRET` - AES cipher secret; removed from env after reading (unless compiled with `keep_env_secret` feature)
- `$USER` - Substituted into DB URL where `$USER` appears

### Config File Format

Lines parsed as `key: value` or `key= value`. Blank lines and lines starting with `#`, `;`, `/`, `[` are ignored.

```
db: postgres://host/dbname
config: myschema.settings
uuid: 550e8400-e29b-41d4-a716-446655440000
token: get-token.sh
ttl: 15
pk: /path/to/private.key
```

### Fields

```rust
pub struct ArgConfig {
    pub uuid: Uuid,              // instance ID (auto-generated if not provided)
    pub uuid_gen: bool,          // true if uuid was auto-generated
    pub db_url: String,          // database connection string
    pub table: String,           // schema.table for config lookup
    pub cfg: HashMap<String, String>, // key=value pairs from config file
    pub token: Token,            // token script + TTL for dynamic DB passwords
    pub pk: Option<KeyFile>,     // RSA/AES private key file
    pub secret: Option<String>,  // AES cipher secret
    pub verbose: Opt,            // -v / --verbose / --debug
    pub quiet: Opt,              // -q / --quiet / --batch
    pub help: Opt,               // -h / --help / --info
    // -V / --version: prints version and exits inside from_args()
}
```

### Usage

```rust
let config = ArgConfig::from_args2(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))?;

// -V / --version is handled inside from_args() - prints "<name> <version>" and exits automatically
// pass your own crate's name/version, since env!() inside marg would resolve to marg's own

if config.help.is_set() {
    println!("Usage: myapp [OPTIONS] [db-url] [schema.table] [uuid] [token] [ttl]");
    return Ok(());
}

let url = config.db_url(); // resolves $PWD placeholder with token value
```

