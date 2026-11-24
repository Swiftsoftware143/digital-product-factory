//! Headless MCP door: the app becomes drivable by an agent.
//!
//! `dpf mcp` speaks MCP over stdio (the form every desktop MCP client understands) and
//! `dpf serve --bind 127.0.0.1:PORT --key-file PATH` answers the same door over HTTP on loopback.
//! Both go through `mcp-door`, the fleet's one transport/credential/audit crate, and both call the
//! SAME engine functions the GUI calls — see `engine.rs` for the mapping.
//!
//! Safety rules encoded here, not documented and hoped for:
//!
//! * `serve` refuses any address that is not loopback, and refuses to start without a key file.
//! * The bearer token is compared in constant time; it is never logged, echoed or written to the
//!   audit line.
//! * In stdio mode the process boundary IS the credential (the client already owns the user's
//!   account), so a per-process token is used and never leaves the process.
//! * Tools that publish, deliver a paid product, charge money or touch the licence are absent
//!   entirely — see the README's "Driving DPF from an agent".

mod engine;
mod settings;

use axum::http::{header, HeaderMap, HeaderValue};
use engine::Engine;
use mcp_door::credential::constant_time_eq;
use mcp_door::{AuthError, Caller, CredentialResolver, Door, Role};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

/// What the model is told once, at `initialize`. Kept short: a client pays for this on every
/// conversation, and the tool descriptions carry the detail.
const INSTRUCTIONS: &str = "\
Digital Product Factory (DPF) — the user's own desktop app. You can read their products \
(pipeline projects), create a product, list product templates, generate product content and run a \
strategy brief. Generation and strategy use the USER'S OWN AI key (BYOK) and spend their provider \
credit, so ask before calling them. Nothing here publishes a listing, delivers a paid product, \
charges money or touches the licence — those tools do not exist on this door.";

/// The subject recorded in the audit log. Never the token, never a token prefix.
const AUDIT_SUBJECT: &str = "local-agent";

pub fn run_stdio(args: &[String]) -> Result<(), String> {
    let opts = Options::parse(args)?;
    let rt = Arc::new(runtime()?);
    let prepared = Prepared::build(&opts, stdio_token(&opts)?, Arc::clone(&rt))?;

    crate::log_line(&format!(
        "mcp: stdio door up — {} tool(s), db {}, audit {}",
        prepared.tool_count,
        prepared.db_path.display(),
        prepared.audit_path.display()
    ));

    let door = Arc::clone(&prepared.door);
    let headers = bearer_headers(&prepared.token)?;

    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut line = String::new();

    loop {
        line.clear();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // the client closed the pipe: a normal end
            Ok(_) => {}
            Err(e) => return Err(format!("cannot read stdin: {e}")),
        }
        if line.trim().is_empty() {
            continue;
        }

        // One newline-delimited JSON-RPC message in, one out. Notifications produce no body.
        let response = rt.block_on(door.dispatch(&headers, line.trim().as_bytes()));
        let bytes = rt
            .block_on(axum::body::to_bytes(response.into_body(), 4 * 1024 * 1024))
            .map_err(|e| format!("cannot read the door's response: {e}"))?;
        if !bytes.is_empty() {
            stdout
                .write_all(&bytes)
                .and_then(|_| stdout.write_all(b"\n"))
                .and_then(|_| stdout.flush())
                .map_err(|e| format!("cannot write to stdout: {e}"))?;
        }
    }

    crate::log_line("mcp: stdio door closed (client ended the stream)");
    Ok(())
}

pub fn run_serve(args: &[String]) -> Result<(), String> {
    let opts = Options::parse(args)?;

    let key_file = opts.key_file.as_deref().ok_or(
        "`serve` needs --key-file PATH: a file whose first non-comment line is the bearer token",
    )?;
    let token = read_key_file(key_file)?;

    let bind = opts
        .bind
        .clone()
        .unwrap_or_else(|| "127.0.0.1:8765".to_string());
    let addr: std::net::SocketAddr = bind
        .parse()
        .map_err(|e| format!("--bind {bind:?} is not an address: {e}"))?;
    if !addr.ip().is_loopback() {
        return Err(format!(
            "refusing to bind {addr}: this door is loopback-only. Use 127.0.0.1:{port} or [::1]:{port}.",
            port = addr.port()
        ));
    }

    let rt = Arc::new(runtime()?);
    let prepared = Prepared::build(&opts, token, Arc::clone(&rt))?;
    crate::log_line(&format!(
        "mcp: http door binding {addr} — {} tool(s), db {}, audit {}",
        prepared.tool_count,
        prepared.db_path.display(),
        prepared.audit_path.display()
    ));

    let door = Arc::clone(&prepared.door);
    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| format!("cannot bind {addr}: {e}"))?;
        // stderr, so a client wrapping this process never sees a stray byte on stdout.
        eprintln!(
            "dpf mcp door listening on http://{addr}{path}",
            path = door.config().path
        );
        axum::serve(listener, door.router())
            .await
            .map_err(|e| format!("server stopped: {e}"))
    })
}

/// Everything both transports need, built once so the two paths cannot drift.
struct Prepared {
    door: Arc<Door>,
    token: String,
    db_path: PathBuf,
    audit_path: PathBuf,
    tool_count: usize,
}

impl Prepared {
    fn build(
        opts: &Options,
        token: String,
        rt: Arc<tokio::runtime::Runtime>,
    ) -> Result<Self, String> {
        let db_path = opts
            .db
            .clone()
            .unwrap_or_else(crate::database::Database::data_file_path);
        let db = Arc::new(
            crate::database::Database::open_at(&db_path)
                .map_err(|e| format!("cannot open database {}: {e}", db_path.display()))?,
        );

        let loaded = settings::load(opts.settings.as_deref());
        crate::log_line(&format!(
            "mcp: settings from {} — {} provider key(s) configured",
            loaded.source, loaded.providers_configured
        ));

        let audit_path = opts
            .audit
            .clone()
            .unwrap_or_else(|| beside(&db_path, "mcp-audit.log"));
        if let Some(dir) = audit_path.parent() {
            if !dir.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(dir);
            }
        }

        let engine = Arc::new(Engine::new(db, rt, loaded.config));
        let registry = engine.registry();
        let tool_count = registry.len();

        let resolver: Arc<dyn CredentialResolver> = Arc::new(KeyResolver {
            token: token.clone(),
        });

        let door = Door::builder(
            "digital-product-factory",
            env!("CARGO_PKG_VERSION"),
            registry,
            resolver,
        )
        .path("/mcp")
        .audit(&audit_path)
        // A strategy brief or a long generation can take a minute; the crate's 30s default would
        // cut it off and report "tool timed out" for a call that was working.
        .tool_timeout(Duration::from_secs(300))
        .instructions(INSTRUCTIONS)
        .build()
        .map_err(|e| format!("the door refused to start: {}", e.0.join("; ")))?;

        Ok(Self {
            door: Arc::new(door),
            token,
            db_path,
            audit_path,
            tool_count,
        })
    }
}

/// The one credential check. Constant time, and it maps to the operator role because the holder
/// of this token IS the person whose data this is.
struct KeyResolver {
    token: String,
}

#[async_trait::async_trait]
impl CredentialResolver for KeyResolver {
    async fn resolve(&self, bearer: &str) -> Result<Caller, AuthError> {
        if constant_time_eq(bearer.as_bytes(), self.token.as_bytes()) {
            Ok(Caller::new(AUDIT_SUBJECT, Role::Admin))
        } else {
            Err(AuthError::Invalid)
        }
    }
}

struct Options {
    db: Option<PathBuf>,
    settings: Option<PathBuf>,
    key_file: Option<PathBuf>,
    bind: Option<String>,
    audit: Option<PathBuf>,
}

impl Options {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut out = Options {
            db: None,
            settings: None,
            key_file: None,
            bind: None,
            audit: None,
        };
        let mut i = 0;
        while i < args.len() {
            let flag = args[i].as_str();
            let value = args.get(i + 1);
            match flag {
                "--db" => out.db = Some(path_arg(flag, value)?),
                "--settings" => out.settings = Some(path_arg(flag, value)?),
                "--key-file" => out.key_file = Some(path_arg(flag, value)?),
                "--audit" => out.audit = Some(path_arg(flag, value)?),
                "--bind" => {
                    out.bind = Some(
                        value
                            .ok_or_else(|| {
                                "--bind needs an address, e.g. 127.0.0.1:8765".to_string()
                            })?
                            .clone(),
                    )
                }
                // The renderer flags belong to the GUI; ignore anything we do not know rather than
                // fail a client that passes extra arguments.
                _ => {
                    i += 1;
                    continue;
                }
            }
            i += 2;
        }
        Ok(out)
    }
}

fn path_arg(flag: &str, value: Option<&String>) -> Result<PathBuf, String> {
    value
        .map(PathBuf::from)
        .ok_or_else(|| format!("{flag} needs a path"))
}

/// stdio: use the key file if given (the token then never changes between runs), otherwise a
/// fresh per-process token. It is only ever compared against itself, so the value is irrelevant —
/// what matters is that the door still takes a real credential path instead of a bypass arm.
fn stdio_token(opts: &Options) -> Result<String, String> {
    match opts.key_file.as_deref() {
        Some(path) => read_key_file(path),
        None => {
            use rand::Rng;
            Ok(rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(40)
                .map(char::from)
                .collect())
        }
    }
}

/// Read the bearer token from a key file.
///
/// Rules (documented in the README, enforced here):
/// * the first non-empty line that does not start with `#` is the token, trimmed;
/// * at least 24 characters — a short "password" on a local port is still a password;
/// * on Unix the file should be `0600`; anything looser is logged as a warning (the door still
///   starts, because on Windows the mode bits do not exist and refusing would lock those users
///   out of their own app).
fn read_key_file(path: &Path) -> Result<String, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read key file {}: {e}", path.display()))?;
    let token = text
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && !l.starts_with('#'))
        .ok_or_else(|| {
            format!(
                "key file {} has no token: its first non-empty, non-comment line must be the bearer token",
                path.display()
            )
        })?;
    if token.len() < 24 {
        return Err(format!(
            "the token in {} is only {} characters; use at least 24 (e.g. `openssl rand -hex 24`)",
            path.display(),
            token.len()
        ));
    }
    if token.chars().any(|c| c.is_whitespace()) {
        return Err(format!(
            "the token in {} contains whitespace; it must be a single line",
            path.display()
        ));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(meta) = std::fs::metadata(path) {
            if meta.mode() & 0o077 != 0 {
                let warning = format!(
                    "mcp: key file {} is readable by other users (mode {:o}); chmod 600 it",
                    path.display(),
                    meta.mode() & 0o777
                );
                crate::log_line(&warning);
                eprintln!("dpf: warning: {warning}");
            }
        }
    }

    Ok(token.to_string())
}

fn bearer_headers(token: &str) -> Result<HeaderMap, String> {
    let mut headers = HeaderMap::new();
    let value = HeaderValue::from_str(&format!("Bearer {token}")).map_err(|_| {
        "the key file's token contains characters an HTTP header cannot carry".to_string()
    })?;
    headers.insert(header::AUTHORIZATION, value);
    Ok(headers)
}

fn runtime() -> Result<tokio::runtime::Runtime, String> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("cannot start the async runtime: {e}"))
}

fn beside(path: &Path, name: &str) -> PathBuf {
    match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir.join(name),
        _ => PathBuf::from(name),
    }
}
