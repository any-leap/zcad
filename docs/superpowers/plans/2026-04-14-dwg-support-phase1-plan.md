# plangen DWG 支持 Phase 1 · 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let plangen read DWG files (incl. AutoCAD 2025 AC1036) by shipping a rust sidecar (`zcad-dwg-service`) that wraps ODA File Converter behind an HTTP contract, then wire plangen's `DrawingAnalyzer` to use it.

**Architecture:** New rust crate in zcad workspace exposing `POST /convert` (multipart DWG → DXF text body). Phase 1 impl shells out to ODA File Converter via `xvfb-run`. Phase 2 will swap in a native rust parser without changing the HTTP contract. plangen gets a new TanStack Start API route that proxies to the sidecar over a private docker network, and reuses the existing `dxf-parser` text-extraction path on the returned DXF.

**Tech Stack:**
- Rust 1.83, axum 0.7, tokio, async-trait, tempfile, thiserror, tracing
- Ubuntu 22.04 + xvfb + ODA File Converter (closed-source binary)
- TanStack Start API routes (`createFileRoute` with `server.handlers`)
- Existing plangen stack: React 19, bun, dxf-parser

**Spec:** `docs/superpowers/specs/2026-04-14-dwg-support-phase1-design.md`

---

## Prerequisites (must be arranged before Task 13)

1. **ODA File Converter .deb URL** — user must register at https://www.opendesign.com/guestfiles/oda_file_converter, download the Linux x64 Qt6 .deb, and upload it to a location with a stable URL (e.g. GitHub release asset in a private repo, or private S3). Save this URL as `ODA_DEB_URL` GitHub Actions secret in `any-leap` org.
2. **Sample DWG fixtures** — user must provide or produce:
   - `sample_2018.dwg` — a minimal DWG saved in ACAD2018 format that contains at least one TEXT entity with a known string (e.g. `"HELLO_2018"`).
   - `sample_2025.dwg` — same content, saved from AutoCAD 2025 (AC1036 format).
   - `corrupt.dwg` — any file whose first bytes are not valid DWG magic (e.g. `head -c 128 /dev/urandom > corrupt.dwg`).

---

## File Structure

### `zcad` repo (sidecar)

| File | Status | Responsibility |
|---|---|---|
| `Cargo.toml` | modify | Add `zcad-dwg-service` to workspace members and shared deps |
| `crates/zcad-dwg-service/Cargo.toml` | create | Crate manifest |
| `crates/zcad-dwg-service/src/main.rs` | create | axum bootstrap, env config, tracing init, backend selection |
| `crates/zcad-dwg-service/src/routes.rs` | create | `POST /convert`, `GET /healthz` handlers |
| `crates/zcad-dwg-service/src/converter.rs` | create | `trait DwgConverter` + `ConvertError` enum |
| `crates/zcad-dwg-service/src/oda_fc.rs` | create | `OdaFcConverter` Phase 1 shell-out impl |
| `crates/zcad-dwg-service/src/native.rs` | create | `NativeConverter` Phase 2 stub |
| `crates/zcad-dwg-service/src/magic.rs` | create | DWG magic-byte validator |
| `crates/zcad-dwg-service/tests/convert_integration.rs` | create | End-to-end HTTP test against real ODA FC |
| `crates/zcad-dwg-service/tests/fixtures/*.dwg` | create | Test fixtures (user-provided, see Prerequisites) |
| `crates/zcad-dwg-service/Dockerfile` | create | Two-stage build: rust → ubuntu+xvfb+ODA FC |
| `.github/workflows/dwg-service.yml` | create | CI: build + push image to ghcr.io |
| `README.md` | modify | Document sidecar build & deploy |

### `plangen` repo (integration)

| File | Status | Responsibility |
|---|---|---|
| `src/lib/drawing-analysis/dxf-reader.ts` | modify | Extract `extractDxfTextFromString` as exported pure function |
| `src/lib/drawing-analysis/dwg-reader.ts` | create | `extractDwgText(file)` — calls API, reuses DXF extractor |
| `src/server/dwg-converter-client.ts` | create | Server-side fetch wrapper that POSTs to sidecar |
| `src/routes/api/convert-dwg.tsx` | create | TanStack Start API route — proxies to sidecar |
| `src/components/editor/DrawingAnalyzer.tsx` | modify | Accept `.dwg`, branch by extension, update loading UI |
| `docker-compose.yml` | modify | Add `dwg-converter` service + internal network |
| `.env.example` | modify | Document `DWG_CONVERTER_URL` |
| `README.md` | modify | Document DWG support and compose service |

---

# Part A — zcad-dwg-service (rust sidecar)

## Task 1: Create crate skeleton

**Files:**
- Create: `crates/zcad-dwg-service/Cargo.toml`
- Create: `crates/zcad-dwg-service/src/main.rs`
- Modify: `Cargo.toml` (workspace root)

- [ ] **Step 1: Add crate to workspace members**

Edit root `Cargo.toml`, in the `[workspace] members` array add `"crates/zcad-dwg-service"`:

```toml
[workspace]
resolver = "2"
members = [
    "crates/zcad-core",
    "crates/zcad-renderer",
    "crates/zcad-file",
    "crates/zcad-ui",
    "crates/zcad-app",
    "crates/zcad-dwg-service",
]
```

- [ ] **Step 2: Add shared dependencies to workspace**

In root `Cargo.toml` under `[workspace.dependencies]`, add (if not already present):

```toml
# HTTP 服务（zcad-dwg-service 使用）
axum = { version = "0.7", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["limit", "trace"] }
async-trait = "0.1"
bytes = "1"
tempfile = "3"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

(If `thiserror`, `tracing`, `tokio` already exist at workspace level, don't re-add — just reuse.)

- [ ] **Step 3: Create `crates/zcad-dwg-service/Cargo.toml`**

```toml
[package]
name = "zcad-dwg-service"
description = "HTTP sidecar that converts DWG to DXF. Phase 1 wraps ODA File Converter."
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
axum.workspace = true
tokio.workspace = true
tower.workspace = true
tower-http.workspace = true
async-trait.workspace = true
bytes.workspace = true
tempfile.workspace = true
thiserror.workspace = true
tracing.workspace = true
tracing-subscriber.workspace = true
serde = { workspace = true, features = ["derive"] }
serde_json.workspace = true

[dev-dependencies]
reqwest = { version = "0.12", features = ["multipart"] }

[features]
# Integration tests require a real ODA File Converter on PATH (or in Docker).
# Enable with: cargo test -p zcad-dwg-service --features integration
integration = []
```

- [ ] **Step 4: Create `crates/zcad-dwg-service/src/main.rs` with empty bin**

```rust
fn main() {
    println!("zcad-dwg-service placeholder");
}
```

- [ ] **Step 5: Verify workspace builds**

Run: `cargo build -p zcad-dwg-service`
Expected: compiles successfully, one warning about unused deps is acceptable.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml crates/zcad-dwg-service/
git commit -m "feat(dwg-service): scaffold crate in workspace"
```

---

## Task 2: Define `DwgConverter` trait and `ConvertError`

**Files:**
- Create: `crates/zcad-dwg-service/src/converter.rs`

- [ ] **Step 1: Write the trait and error type**

Create `crates/zcad-dwg-service/src/converter.rs`:

```rust
use async_trait::async_trait;
use bytes::Bytes;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("input is not a DWG file (bad magic bytes)")]
    InvalidMagic,

    #[error("input too large: {size} bytes (max {max})")]
    TooLarge { size: usize, max: usize },

    #[error("ODA File Converter failed (exit {code:?}): {stderr}")]
    OdaFailed { code: Option<i32>, stderr: String },

    #[error("conversion timed out after {secs} seconds")]
    Timeout { secs: u64 },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("internal error: {0}")]
    Internal(String),
}

impl ConvertError {
    /// Map to HTTP status code for the /convert endpoint.
    pub fn status_code(&self) -> u16 {
        match self {
            ConvertError::InvalidMagic => 415,
            ConvertError::TooLarge { .. } => 413,
            ConvertError::OdaFailed { .. } => 422,
            ConvertError::Timeout { .. } => 504,
            ConvertError::Io(_) | ConvertError::Internal(_) => 500,
        }
    }

    pub fn code_str(&self) -> &'static str {
        match self {
            ConvertError::InvalidMagic => "invalid_magic",
            ConvertError::TooLarge { .. } => "too_large",
            ConvertError::OdaFailed { .. } => "conversion_failed",
            ConvertError::Timeout { .. } => "timeout",
            ConvertError::Io(_) => "io_error",
            ConvertError::Internal(_) => "internal",
        }
    }
}

#[async_trait]
pub trait DwgConverter: Send + Sync {
    async fn convert(&self, dwg: Bytes) -> Result<String, ConvertError>;
}
```

- [ ] **Step 2: Register module in `main.rs`**

Replace `src/main.rs` contents with:

```rust
mod converter;

fn main() {
    println!("zcad-dwg-service placeholder");
}
```

- [ ] **Step 3: Build**

Run: `cargo build -p zcad-dwg-service`
Expected: compiles cleanly.

- [ ] **Step 4: Commit**

```bash
git add crates/zcad-dwg-service/src/converter.rs crates/zcad-dwg-service/src/main.rs
git commit -m "feat(dwg-service): add DwgConverter trait and ConvertError"
```

---

## Task 3: DWG magic-byte validator

**Files:**
- Create: `crates/zcad-dwg-service/src/magic.rs`

- [ ] **Step 1: Write failing unit test**

Create `crates/zcad-dwg-service/src/magic.rs`:

```rust
/// DWG files start with ASCII "AC" followed by 4 ASCII digits identifying version.
/// Examples: "AC1015" (2000), "AC1032" (2018), "AC1036" (2025).
pub fn is_dwg_magic(bytes: &[u8]) -> bool {
    bytes.len() >= 6
        && &bytes[0..2] == b"AC"
        && bytes[2..6].iter().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ac1032() {
        assert!(is_dwg_magic(b"AC1032\x00\x00"));
    }

    #[test]
    fn accepts_ac1036() {
        assert!(is_dwg_magic(b"AC1036extra"));
    }

    #[test]
    fn rejects_short() {
        assert!(!is_dwg_magic(b"AC10"));
    }

    #[test]
    fn rejects_non_ac() {
        assert!(!is_dwg_magic(b"PK\x03\x04aaa"));
    }

    #[test]
    fn rejects_non_digit() {
        assert!(!is_dwg_magic(b"ACxxxx"));
    }
}
```

- [ ] **Step 2: Register module in `main.rs`**

Add `mod magic;` below `mod converter;` in `src/main.rs`.

- [ ] **Step 3: Run tests**

Run: `cargo test -p zcad-dwg-service magic`
Expected: 5 passed.

- [ ] **Step 4: Commit**

```bash
git add crates/zcad-dwg-service/src/magic.rs crates/zcad-dwg-service/src/main.rs
git commit -m "feat(dwg-service): add DWG magic-byte validator"
```

---

## Task 4: Phase 2 placeholder `NativeConverter`

**Files:**
- Create: `crates/zcad-dwg-service/src/native.rs`

- [ ] **Step 1: Write stub impl**

```rust
use async_trait::async_trait;
use bytes::Bytes;

use crate::converter::{ConvertError, DwgConverter};

/// Phase 2 seam. Will be implemented with a native rust DWG parser
/// (likely built on zcad-file). Currently returns an error so that
/// selecting this backend in Phase 1 is a loud failure, not silent.
pub struct NativeConverter;

#[async_trait]
impl DwgConverter for NativeConverter {
    async fn convert(&self, _dwg: Bytes) -> Result<String, ConvertError> {
        Err(ConvertError::Internal(
            "native backend not implemented (Phase 2)".into(),
        ))
    }
}
```

- [ ] **Step 2: Register module**

Add `mod native;` in `src/main.rs`.

- [ ] **Step 3: Build**

Run: `cargo build -p zcad-dwg-service`
Expected: compiles.

- [ ] **Step 4: Commit**

```bash
git add crates/zcad-dwg-service/src/native.rs crates/zcad-dwg-service/src/main.rs
git commit -m "feat(dwg-service): add NativeConverter Phase 2 stub"
```

---

## Task 5: `OdaFcConverter` — magic check + temp dirs

Build the ODA FC impl incrementally. This task does steps 1-2 (magic check + temp dir scaffolding); Task 6 adds the actual shell-out.

**Files:**
- Create: `crates/zcad-dwg-service/src/oda_fc.rs`

- [ ] **Step 1: Write struct with config**

```rust
use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use tempfile::TempDir;
use tokio::fs;

use crate::converter::{ConvertError, DwgConverter};
use crate::magic::is_dwg_magic;

pub struct OdaFcConverter {
    pub binary: PathBuf,
    pub timeout: Duration,
}

impl OdaFcConverter {
    pub fn new(binary: impl Into<PathBuf>, timeout: Duration) -> Self {
        Self {
            binary: binary.into(),
            timeout,
        }
    }

    async fn prepare_input(&self, dwg: &Bytes) -> Result<(TempDir, PathBuf), ConvertError> {
        if !is_dwg_magic(dwg) {
            return Err(ConvertError::InvalidMagic);
        }

        let tmp = tempfile::tempdir()?;
        let in_dir = tmp.path().join("in");
        let out_dir = tmp.path().join("out");
        fs::create_dir(&in_dir).await?;
        fs::create_dir(&out_dir).await?;
        let in_file = in_dir.join("input.dwg");
        fs::write(&in_file, dwg).await?;
        Ok((tmp, tmp_path(&tmp, "in"))) // will refine in Task 6
    }
}

fn tmp_path(tmp: &TempDir, sub: &str) -> PathBuf {
    tmp.path().join(sub)
}

#[async_trait]
impl DwgConverter for OdaFcConverter {
    async fn convert(&self, _dwg: Bytes) -> Result<String, ConvertError> {
        Err(ConvertError::Internal("not implemented yet".into()))
    }
}
```

- [ ] **Step 2: Register module**

Add `mod oda_fc;` in `src/main.rs`.

- [ ] **Step 3: Build**

Run: `cargo build -p zcad-dwg-service`
Expected: compiles (Task 6 will fill in `convert`).

- [ ] **Step 4: Commit**

```bash
git add crates/zcad-dwg-service/src/oda_fc.rs crates/zcad-dwg-service/src/main.rs
git commit -m "feat(dwg-service): OdaFcConverter scaffolding"
```

---

## Task 6: `OdaFcConverter` — shell-out with timeout

**Files:**
- Modify: `crates/zcad-dwg-service/src/oda_fc.rs`

- [ ] **Step 1: Write failing unit test for magic rejection**

Append to `oda_fc.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn rejects_non_dwg_input() {
        let converter = OdaFcConverter::new("/nonexistent", Duration::from_secs(5));
        let result = converter.convert(Bytes::from_static(b"not a dwg")).await;
        assert!(matches!(result, Err(ConvertError::InvalidMagic)));
    }
}
```

- [ ] **Step 2: Run test — expect it to fail**

Run: `cargo test -p zcad-dwg-service oda_fc::tests::rejects_non_dwg_input`
Expected: FAIL — current `convert` returns `Internal`, not `InvalidMagic`.

- [ ] **Step 3: Implement full `convert`**

Replace the entire `oda_fc.rs` with:

```rust
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use tempfile::TempDir;
use tokio::fs;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::converter::{ConvertError, DwgConverter};
use crate::magic::is_dwg_magic;

pub struct OdaFcConverter {
    pub binary: PathBuf,
    pub timeout: Duration,
}

impl OdaFcConverter {
    pub fn new(binary: impl Into<PathBuf>, timeout: Duration) -> Self {
        Self {
            binary: binary.into(),
            timeout,
        }
    }
}

#[async_trait]
impl DwgConverter for OdaFcConverter {
    async fn convert(&self, dwg: Bytes) -> Result<String, ConvertError> {
        if !is_dwg_magic(&dwg) {
            return Err(ConvertError::InvalidMagic);
        }

        let tmp: TempDir = tempfile::tempdir()?;
        let in_dir = tmp.path().join("in");
        let out_dir = tmp.path().join("out");
        fs::create_dir(&in_dir).await?;
        fs::create_dir(&out_dir).await?;
        let in_file = in_dir.join("input.dwg");
        fs::write(&in_file, &dwg).await?;

        debug!(
            bytes = dwg.len(),
            in_dir = %in_dir.display(),
            "running ODA File Converter"
        );

        // ODAFileConverter <in_dir> <out_dir> <out_ver> <out_fmt> <recursive> <audit> <filter>
        // 0 = non-recursive, 1 = audit, *.dwg = filter
        let mut cmd = Command::new(&self.binary);
        cmd.arg(&in_dir)
            .arg(&out_dir)
            .arg("ACAD2018")
            .arg("DXF")
            .arg("0")
            .arg("1")
            .arg("*.dwg")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let child = cmd.spawn().map_err(|e| {
            ConvertError::Internal(format!("failed to spawn {}: {}", self.binary.display(), e))
        })?;

        let output = match timeout(self.timeout, child.wait_with_output()).await {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => return Err(ConvertError::Io(e)),
            Err(_) => {
                warn!("ODA FC timed out");
                return Err(ConvertError::Timeout {
                    secs: self.timeout.as_secs(),
                });
            }
        };

        if !output.status.success() {
            return Err(ConvertError::OdaFailed {
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        // ODA FC names the output same as input, extension swapped.
        let out_file = out_dir.join("input.dxf");
        let dxf = fs::read_to_string(&out_file).await.map_err(|e| {
            ConvertError::OdaFailed {
                code: output.status.code(),
                stderr: format!("ODA FC exited 0 but output DXF missing: {}", e),
            }
        })?;

        // tmp drops here, cleaning up both dirs.
        Ok(dxf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_non_dwg_input() {
        let converter = OdaFcConverter::new("/nonexistent", Duration::from_secs(5));
        let result = converter.convert(Bytes::from_static(b"not a dwg")).await;
        assert!(matches!(result, Err(ConvertError::InvalidMagic)));
    }

    #[tokio::test]
    async fn fails_gracefully_when_binary_missing() {
        let converter = OdaFcConverter::new("/nonexistent/oda", Duration::from_secs(5));
        let dwg = Bytes::from_static(b"AC1032\x00\x00padding");
        let result = converter.convert(dwg).await;
        assert!(matches!(result, Err(ConvertError::Internal(_))));
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p zcad-dwg-service oda_fc`
Expected: 2 passed.

- [ ] **Step 5: Commit**

```bash
git add crates/zcad-dwg-service/src/oda_fc.rs
git commit -m "feat(dwg-service): implement OdaFcConverter shell-out with timeout"
```

---

## Task 7: HTTP routes — `/healthz` and `/convert`

**Files:**
- Create: `crates/zcad-dwg-service/src/routes.rs`

- [ ] **Step 1: Create routes module**

```rust
use std::sync::Arc;

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use bytes::Bytes;
use serde_json::json;
use tracing::{error, info};

use crate::converter::{ConvertError, DwgConverter};

pub struct AppState {
    pub converter: Arc<dyn DwgConverter>,
}

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/convert", post(convert))
        .with_state(state)
}

async fn healthz() -> &'static str {
    "ok"
}

async fn convert(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Response {
    let dwg_bytes = match read_file_field(&mut multipart).await {
        Ok(b) => b,
        Err(resp) => return resp,
    };

    let size = dwg_bytes.len();
    info!(size, "received DWG for conversion");

    match state.converter.convert(dwg_bytes).await {
        Ok(dxf) => {
            info!(dxf_bytes = dxf.len(), "conversion ok");
            (
                StatusCode::OK,
                [("content-type", "application/vnd.dxf; charset=utf-8")],
                dxf,
            )
                .into_response()
        }
        Err(e) => {
            error!(error = %e, "conversion failed");
            error_response(e)
        }
    }
}

async fn read_file_field(mp: &mut Multipart) -> Result<Bytes, Response> {
    while let Some(field) = mp.next_field().await.map_err(multipart_err)? {
        if field.name() == Some("file") {
            return field.bytes().await.map_err(multipart_err);
        }
    }
    Err((
        StatusCode::BAD_REQUEST,
        json_body("missing_file", "multipart field 'file' is required"),
    )
        .into_response())
}

fn multipart_err(e: axum::extract::multipart::MultipartError) -> Response {
    (
        StatusCode::BAD_REQUEST,
        json_body("bad_multipart", &e.to_string()),
    )
        .into_response()
}

fn error_response(e: ConvertError) -> Response {
    let status = StatusCode::from_u16(e.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    (status, json_body(e.code_str(), &e.to_string())).into_response()
}

fn json_body(code: &str, message: &str) -> ([(&'static str, &'static str); 1], String) {
    (
        [("content-type", "application/json")],
        json!({ "error": code, "message": message }).to_string(),
    )
}
```

- [ ] **Step 2: Register module**

Add `mod routes;` to `src/main.rs`.

- [ ] **Step 3: Build**

Run: `cargo build -p zcad-dwg-service`
Expected: compiles.

- [ ] **Step 4: Commit**

```bash
git add crates/zcad-dwg-service/src/routes.rs crates/zcad-dwg-service/src/main.rs
git commit -m "feat(dwg-service): add /healthz and /convert HTTP routes"
```

---

## Task 8: `main.rs` — wire server, env config, backend selection

**Files:**
- Modify: `crates/zcad-dwg-service/src/main.rs`

- [ ] **Step 1: Replace `main.rs` with full bootstrap**

```rust
mod converter;
mod magic;
mod native;
mod oda_fc;
mod routes;

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

use crate::converter::DwgConverter;
use crate::native::NativeConverter;
use crate::oda_fc::OdaFcConverter;
use crate::routes::{router, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use stderr so conversion output on stdout stays clean if ever piped.
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_writer(std::io::stderr)
        .init();

    let port: u16 = env::var("PORT").ok().and_then(|s| s.parse().ok()).unwrap_or(8080);
    let max_bytes: usize = env::var("ZCAD_DWG_MAX_BYTES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50 * 1024 * 1024);
    let timeout_secs: u64 = env::var("ZCAD_DWG_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let backend = env::var("ZCAD_DWG_BACKEND").unwrap_or_else(|_| "oda".into());
    let oda_bin: PathBuf = env::var("ZCAD_DWG_ODA_BIN")
        .unwrap_or_else(|_| "ODAFileConverter".into())
        .into();

    let converter: Arc<dyn DwgConverter> = match backend.as_str() {
        "oda" => Arc::new(OdaFcConverter::new(oda_bin, Duration::from_secs(timeout_secs))),
        "native" => Arc::new(NativeConverter),
        other => anyhow::bail!("unknown ZCAD_DWG_BACKEND: {}", other),
    };

    let state = Arc::new(AppState { converter });

    let app = router(state)
        .layer(RequestBodyLimitLayer::new(max_bytes))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(%addr, backend, max_bytes, timeout_secs, "zcad-dwg-service starting");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

- [ ] **Step 2: Add `anyhow` dep**

In `crates/zcad-dwg-service/Cargo.toml` `[dependencies]`, add:

```toml
anyhow = "1"
```

- [ ] **Step 3: Build**

Run: `cargo build -p zcad-dwg-service`
Expected: compiles.

- [ ] **Step 4: Smoke test health endpoint**

Run in one terminal: `ZCAD_DWG_ODA_BIN=/bin/true cargo run -p zcad-dwg-service`
Run in another: `curl -fsS http://localhost:8080/healthz`
Expected: `ok`. Ctrl-C the server after.

- [ ] **Step 5: Commit**

```bash
git add crates/zcad-dwg-service/src/main.rs crates/zcad-dwg-service/Cargo.toml
git commit -m "feat(dwg-service): wire axum server with env config and backend selection"
```

---

## Task 9: Integration test fixtures

**Files:**
- Create: `crates/zcad-dwg-service/tests/fixtures/sample_2018.dwg` (user-provided)
- Create: `crates/zcad-dwg-service/tests/fixtures/sample_2025.dwg` (user-provided)
- Create: `crates/zcad-dwg-service/tests/fixtures/corrupt.dwg`
- Create: `crates/zcad-dwg-service/tests/fixtures/README.md`

- [ ] **Step 1: Create fixtures directory and corrupt fixture**

```bash
mkdir -p crates/zcad-dwg-service/tests/fixtures
head -c 128 /dev/urandom > crates/zcad-dwg-service/tests/fixtures/corrupt.dwg
```

- [ ] **Step 2: Write fixtures README**

Create `crates/zcad-dwg-service/tests/fixtures/README.md`:

```markdown
# Test fixtures

- `corrupt.dwg` — random bytes, for testing magic-byte rejection.
- `sample_2018.dwg` — **user must provide**. A minimal DWG saved as ACAD2018 format
  containing a single TEXT entity with the string `HELLO_2018`.
- `sample_2025.dwg` — **user must provide**. Same content but saved from AutoCAD 2025
  (AC1036 format) to validate coverage of the newest format.

The two `sample_*.dwg` files gate the Phase 1 acceptance criteria.
```

- [ ] **Step 3: Check in user-provided fixtures**

**ACTION REQUIRED:** Place the two `.dwg` sample files into `crates/zcad-dwg-service/tests/fixtures/` before proceeding.

Verify:
```bash
ls -la crates/zcad-dwg-service/tests/fixtures/
```
Expected: three files (two samples + corrupt.dwg + README.md).

- [ ] **Step 4: Commit**

```bash
git add crates/zcad-dwg-service/tests/fixtures/
git commit -m "test(dwg-service): add DWG fixtures for integration tests"
```

---

## Task 10: Integration test (`--features integration`)

**Files:**
- Create: `crates/zcad-dwg-service/tests/convert_integration.rs`

- [ ] **Step 1: Write integration test**

```rust
#![cfg(feature = "integration")]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use bytes::Bytes;
use reqwest::multipart::{Form, Part};
use tower_http::limit::RequestBodyLimitLayer;

use zcad_dwg_service::{
    converter::DwgConverter, oda_fc::OdaFcConverter, routes::{router, AppState},
};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

async fn spawn_app() -> String {
    let oda_bin: PathBuf = std::env::var("ZCAD_DWG_ODA_BIN")
        .unwrap_or_else(|_| "ODAFileConverter".into())
        .into();
    let converter: Arc<dyn DwgConverter> =
        Arc::new(OdaFcConverter::new(oda_bin, Duration::from_secs(60)));
    let state = Arc::new(AppState { converter });
    let app: Router = router(state).layer(RequestBodyLimitLayer::new(50 * 1024 * 1024));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{}", addr)
}

async fn post_file(base: &str, path: PathBuf) -> reqwest::Response {
    let bytes = tokio::fs::read(&path).await.unwrap();
    let part = Part::bytes(bytes).file_name(path.file_name().unwrap().to_string_lossy().into_owned());
    let form = Form::new().part("file", part);
    reqwest::Client::new()
        .post(format!("{}/convert", base))
        .multipart(form)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn converts_2018_dwg() {
    let base = spawn_app().await;
    let resp = post_file(&base, fixtures_dir().join("sample_2018.dwg")).await;
    assert_eq!(resp.status(), 200);
    let body = resp.text().await.unwrap();
    assert!(body.contains("HELLO_2018"), "DXF should contain text from DWG");
}

#[tokio::test]
async fn converts_2025_dwg() {
    let base = spawn_app().await;
    let resp = post_file(&base, fixtures_dir().join("sample_2025.dwg")).await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn rejects_corrupt_dwg() {
    let base = spawn_app().await;
    let resp = post_file(&base, fixtures_dir().join("corrupt.dwg")).await;
    assert_eq!(resp.status(), 415);
}

#[tokio::test]
async fn healthz_ok() {
    let base = spawn_app().await;
    let resp = reqwest::get(format!("{}/healthz", base)).await.unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.text().await.unwrap(), "ok");
}
```

- [ ] **Step 2: Expose crate internals to integration tests**

The integration test imports `zcad_dwg_service::routes` etc. Change `main.rs` to be a thin wrapper over a library:

Create `crates/zcad-dwg-service/src/lib.rs`:

```rust
pub mod converter;
pub mod magic;
pub mod native;
pub mod oda_fc;
pub mod routes;
```

Remove the `mod ...;` declarations from `main.rs` and replace its top with:

```rust
use zcad_dwg_service::converter::DwgConverter;
use zcad_dwg_service::native::NativeConverter;
use zcad_dwg_service::oda_fc::OdaFcConverter;
use zcad_dwg_service::routes::{router, AppState};
```

(keep the rest of `main.rs` from Task 8 intact).

Update `Cargo.toml` to declare both a lib and bin:

```toml
[lib]
path = "src/lib.rs"

[[bin]]
name = "zcad-dwg-service"
path = "src/main.rs"
```

- [ ] **Step 3: Build**

Run: `cargo build -p zcad-dwg-service`
Expected: compiles clean.

- [ ] **Step 4: Run non-integration tests**

Run: `cargo test -p zcad-dwg-service`
Expected: all unit tests pass; integration tests are skipped (feature off).

- [ ] **Step 5: (If ODA FC installed locally) run integration tests**

Run: `cargo test -p zcad-dwg-service --features integration -- --test-threads=1`
Expected: 4 tests pass. If ODA FC is not installed, **skip this step** — it will run in CI against the Docker image.

- [ ] **Step 6: Commit**

```bash
git add crates/zcad-dwg-service/
git commit -m "test(dwg-service): add HTTP integration tests behind feature flag"
```

---

## Task 11: Dockerfile

**Files:**
- Create: `crates/zcad-dwg-service/Dockerfile`
- Create: `crates/zcad-dwg-service/.dockerignore`

- [ ] **Step 1: Write Dockerfile**

```dockerfile
# syntax=docker/dockerfile:1.6

# --- Build stage: rust ---
FROM rust:1.83-bookworm AS builder
WORKDIR /build
COPY . .
RUN cargo build --release -p zcad-dwg-service

# --- Runtime stage: ubuntu + xvfb + ODA FC ---
FROM ubuntu:22.04

RUN apt-get update && apt-get install -y --no-install-recommends \
      xvfb \
      libxkbcommon0 \
      libglib2.0-0 \
      libgl1 \
      libfontconfig1 \
      libsm6 \
      libxrender1 \
      libxi6 \
      libdbus-1-3 \
      ca-certificates \
      curl \
 && rm -rf /var/lib/apt/lists/*

# ODA File Converter .deb URL is a secret (not in repo) — passed at build time.
ARG ODA_DEB_URL
RUN test -n "$ODA_DEB_URL" || (echo "ERROR: ODA_DEB_URL build-arg is required" && exit 1) && \
    curl -fsSL "$ODA_DEB_URL" -o /tmp/oda.deb && \
    (dpkg -i /tmp/oda.deb || apt-get update && apt-get install -fy --no-install-recommends) && \
    rm /tmp/oda.deb && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/zcad-dwg-service /usr/local/bin/zcad-dwg-service

ENV ZCAD_DWG_BACKEND=oda \
    ZCAD_DWG_MAX_BYTES=52428800 \
    ZCAD_DWG_TIMEOUT_SECS=60 \
    ZCAD_DWG_ODA_BIN=ODAFileConverter \
    PORT=8080 \
    RUST_LOG=info

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD curl -f http://localhost:8080/healthz || exit 1

CMD ["xvfb-run", "-a", "--server-args=-screen 0 1024x768x24", "/usr/local/bin/zcad-dwg-service"]
```

- [ ] **Step 2: Write `.dockerignore`**

```
target/
**/target/
.git/
docs/
*.md
!crates/zcad-dwg-service/tests/fixtures/README.md
```

- [ ] **Step 3: Build the image locally (requires ODA_DEB_URL)**

Run (from zcad workspace root):
```bash
docker build \
  -f crates/zcad-dwg-service/Dockerfile \
  --build-arg ODA_DEB_URL="<your-oda-deb-url>" \
  -t zcad-dwg-service:local \
  .
```
Expected: image builds successfully. **If no ODA_DEB_URL available yet, skip to Task 12 and come back.**

- [ ] **Step 4: Smoke test container**

```bash
docker run --rm -p 8080:8080 zcad-dwg-service:local &
sleep 5
curl -fsS http://localhost:8080/healthz
docker ps | grep zcad-dwg-service | awk '{print $1}' | xargs docker stop
```
Expected: `ok`.

- [ ] **Step 5: Commit**

```bash
git add crates/zcad-dwg-service/Dockerfile crates/zcad-dwg-service/.dockerignore
git commit -m "build(dwg-service): add Dockerfile with xvfb + ODA FC"
```

---

## Task 12: CI — GitHub Actions workflow

**Files:**
- Create: `.github/workflows/dwg-service.yml`

- [ ] **Step 1: Write workflow**

```yaml
name: dwg-service

on:
  push:
    branches: [main]
    paths:
      - "crates/zcad-dwg-service/**"
      - ".github/workflows/dwg-service.yml"
      - "Cargo.toml"
      - "Cargo.lock"
  pull_request:
    paths:
      - "crates/zcad-dwg-service/**"
      - ".github/workflows/dwg-service.yml"

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: any-leap/zcad-dwg-service

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.83
      - uses: Swatinem/rust-cache@v2
      - name: Unit tests
        run: cargo test -p zcad-dwg-service

  build-and-push:
    needs: test
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - name: Login to ghcr.io
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - name: Docker metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=raw,value=latest
            type=sha,format=short
      - name: Build and push
        uses: docker/build-push-action@v6
        with:
          context: .
          file: crates/zcad-dwg-service/Dockerfile
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          build-args: |
            ODA_DEB_URL=${{ secrets.ODA_DEB_URL }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

- [ ] **Step 2: Verify `ODA_DEB_URL` secret exists in the `any-leap/zcad` repo or `any-leap` org**

**ACTION REQUIRED:** In GitHub repo settings → Secrets and variables → Actions, add `ODA_DEB_URL`. Do **not** commit the URL.

- [ ] **Step 3: Commit workflow**

```bash
git add .github/workflows/dwg-service.yml
git commit -m "ci(dwg-service): build and push image to ghcr.io"
```

- [ ] **Step 4: Push and verify CI run**

```bash
git push origin main
```
Open GitHub Actions tab, watch `dwg-service` workflow. Expected: `test` job passes; `build-and-push` produces `ghcr.io/any-leap/zcad-dwg-service:latest` and `...:sha-<short>`.

---

## Task 13: Update zcad README

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Append section to README**

At the end of `README.md`, before any trailing section dividers, add:

```markdown
## DWG 转换服务（zcad-dwg-service）

sidecar HTTP 服务，将 DWG 转为 DXF。Phase 1 内部 shell-out 到 ODA File Converter；
Phase 2 将替换为原生 rust 解析器。

### 接口

- `POST /convert` — multipart/form-data，字段 `file`（DWG），返回 DXF 文本
- `GET /healthz` — 健康检查

### 本地构建镜像

需要 ODA File Converter 的 .deb 下载链接（从 https://www.opendesign.com/ 获取，需要免费账号）：

```bash
docker build \
  -f crates/zcad-dwg-service/Dockerfile \
  --build-arg ODA_DEB_URL="<your-url>" \
  -t zcad-dwg-service:local .
```

### 发布的镜像

`ghcr.io/any-leap/zcad-dwg-service:latest`

### 环境变量

| 变量 | 默认 | 说明 |
|---|---|---|
| `PORT` | 8080 | 监听端口 |
| `ZCAD_DWG_BACKEND` | oda | `oda` 或 `native` |
| `ZCAD_DWG_MAX_BYTES` | 52428800 | 请求 body 上限 |
| `ZCAD_DWG_TIMEOUT_SECS` | 60 | 转换超时 |
| `ZCAD_DWG_ODA_BIN` | `ODAFileConverter` | ODA FC 可执行路径 |
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add zcad-dwg-service section to README"
```

---

# Part B — plangen integration

> All tasks in Part B happen in the `plangen` repo. Cd to `~/developer/plangen` before starting.

## Task 14: Refactor `dxf-reader.ts` to expose string-based extractor

**Files:**
- Modify: `src/lib/drawing-analysis/dxf-reader.ts`

- [ ] **Step 1: Rewrite file with exported pure function**

Replace the entire contents of `src/lib/drawing-analysis/dxf-reader.ts` with:

```ts
// 浏览器端 DXF 文本抽取
// DXF 是 AutoCAD 导出的 ASCII 格式，里面有 TEXT / MTEXT / ATTRIB 三类文字实体。

import DxfParser from "dxf-parser"

type DxfEntity = {
  type: string
  text?: string
  attributes?: Array<{ text?: string }>
}

/**
 * 从已经读入内存的 DXF 字符串抽取所有文字实体的字符串。
 * 这是内核，可被 DXF 直接路径和 DWG 转换路径共用。
 */
export function extractDxfTextFromString(dxfText: string): string {
  const parser = new DxfParser()
  let dxf: ReturnType<DxfParser["parseSync"]>
  try {
    dxf = parser.parseSync(dxfText)
  } catch (e) {
    throw new Error(`DXF 解析失败：${String(e)}`)
  }
  if (!dxf) throw new Error("DXF 内容为空")

  const out: Array<string> = []

  const entities = (dxf as unknown as { entities?: Array<DxfEntity> }).entities ?? []
  for (const ent of entities) {
    if (ent.type === "TEXT" || ent.type === "MTEXT") {
      const raw = ent.text ?? ""
      const clean = stripMtextFormatting(raw)
      if (clean.trim()) out.push(clean)
    }
    if (ent.type === "INSERT" && Array.isArray(ent.attributes)) {
      for (const attr of ent.attributes) {
        const raw = attr.text ?? ""
        if (raw.trim()) out.push(stripMtextFormatting(raw))
      }
    }
  }

  const blocks =
    (dxf as unknown as { blocks?: Record<string, { entities?: Array<DxfEntity> }> }).blocks ?? {}
  for (const blk of Object.values(blocks)) {
    for (const ent of blk.entities ?? []) {
      if (ent.type === "TEXT" || ent.type === "MTEXT") {
        const clean = stripMtextFormatting(ent.text ?? "")
        if (clean.trim()) out.push(clean)
      }
    }
  }

  return out.join("\n")
}

/**
 * 从 File/Blob 读取并抽取（原 dxfout 路径的入口，保留向后兼容）。
 */
export async function extractDxfText(file: File | Blob): Promise<string> {
  const text = await file.text()
  return extractDxfTextFromString(text)
}

/**
 * 去掉 MTEXT 的格式码。
 */
function stripMtextFormatting(raw: string): string {
  return raw
    .replace(/\\[A-Za-z]\S*?;/g, "")
    .replace(/[{}]/g, "")
    .replace(/\\P/g, "\n")
    .replace(/\\~/g, " ")
    .replace(/\\[A-Za-z]/g, "")
    .trim()
}
```

- [ ] **Step 2: Typecheck**

Run: `bun run typecheck`
Expected: passes with no errors (or same errors as before if there was existing baseline).

- [ ] **Step 3: Verify existing DXF callsites still compile**

Run: `bun run typecheck 2>&1 | grep dxf-reader || echo "clean"`
Expected: `clean`.

- [ ] **Step 4: Commit**

```bash
git add src/lib/drawing-analysis/dxf-reader.ts
git commit -m "refactor(dxf): expose extractDxfTextFromString for reuse"
```

---

## Task 15: Server-side sidecar client

**Files:**
- Create: `src/server/dwg-converter-client.ts`

- [ ] **Step 1: Write the client**

```ts
// 调用 zcad-dwg-service sidecar 的服务端 wrapper。
// 只能在 server route handler 里用。

const MAX_BYTES = 50 * 1024 * 1024

export class DwgConvertError extends Error {
  constructor(public readonly status: number, public readonly code: string, message: string) {
    super(message)
  }
}

export async function convertDwgToDxf(file: File): Promise<string> {
  const url = process.env.DWG_CONVERTER_URL
  if (!url) {
    throw new DwgConvertError(500, "not_configured", "DWG_CONVERTER_URL 未配置")
  }
  if (file.size > MAX_BYTES) {
    throw new DwgConvertError(413, "too_large", `DWG 文件超过 ${MAX_BYTES / 1024 / 1024} MB 上限`)
  }

  const form = new FormData()
  form.append("file", file, file.name || "input.dwg")

  let resp: Response
  try {
    resp = await fetch(`${url}/convert`, { method: "POST", body: form })
  } catch (e) {
    throw new DwgConvertError(502, "sidecar_unreachable", `无法连接到转换服务：${String(e)}`)
  }

  if (resp.ok) {
    return await resp.text()
  }

  let code = "unknown"
  let message = `sidecar 返回 ${resp.status}`
  try {
    const json = (await resp.json()) as { error?: string; message?: string }
    if (json.error) code = json.error
    if (json.message) message = json.message
  } catch {
    // 响应不是 JSON，保留默认
  }
  throw new DwgConvertError(resp.status, code, message)
}
```

- [ ] **Step 2: Typecheck**

Run: `bun run typecheck`
Expected: passes.

- [ ] **Step 3: Commit**

```bash
git add src/server/dwg-converter-client.ts
git commit -m "feat(dwg): add server-side sidecar client"
```

---

## Task 16: `/api/convert-dwg` TanStack Start route

**Files:**
- Create: `src/routes/api/convert-dwg.tsx`

- [ ] **Step 1: Write the route**

```tsx
// POST /api/convert-dwg
// multipart/form-data: file (.dwg)
// → 200 text/plain (DXF 文本) 或 4xx JSON { error, message }

import { createFileRoute } from "@tanstack/react-router"
import { convertDwgToDxf, DwgConvertError } from "@/server/dwg-converter-client"
import { json, requireSession } from "@/server/require-auth"

export const Route = createFileRoute("/api/convert-dwg")({
  server: {
    handlers: {
      POST: async ({ request }) => {
        requireSession(request)

        const form = await request.formData()
        const file = form.get("file")
        if (!(file instanceof File)) {
          return json({ error: "bad_request", message: "缺少 file 字段" }, 400)
        }
        if (!file.name.toLowerCase().endsWith(".dwg")) {
          return json({ error: "bad_extension", message: "文件必须是 .dwg" }, 400)
        }

        try {
          const dxf = await convertDwgToDxf(file)
          return new Response(dxf, {
            status: 200,
            headers: { "content-type": "application/vnd.dxf; charset=utf-8" },
          })
        } catch (e) {
          if (e instanceof DwgConvertError) {
            return json({ error: e.code, message: e.message }, e.status)
          }
          return json({ error: "internal", message: String(e) }, 500)
        }
      },
    },
  },
})
```

- [ ] **Step 2: Regenerate route tree**

Run: `bun run dev` once briefly (let it write `routeTree.gen.ts`), then Ctrl-C. Or:
Run: `bun run build` to trigger route generation without starting dev server.
Expected: `src/routeTree.gen.ts` now contains an entry for `/api/convert-dwg`.

- [ ] **Step 3: Typecheck**

Run: `bun run typecheck`
Expected: passes.

- [ ] **Step 4: Commit**

```bash
git add src/routes/api/convert-dwg.tsx src/routeTree.gen.ts
git commit -m "feat(dwg): add /api/convert-dwg route proxying to sidecar"
```

---

## Task 17: `dwg-reader.ts` — browser-side API caller + reuse DXF extractor

**Files:**
- Create: `src/lib/drawing-analysis/dwg-reader.ts`

- [ ] **Step 1: Write the reader**

```ts
// 浏览器端 DWG 读取：上传到 /api/convert-dwg，拿回 DXF 文本后复用 DXF 抽取。

import { extractDxfTextFromString } from "./dxf-reader"

// 按 spec §7.1 的错误码映射到用户可读中文。服务端（来自 sidecar）默认是英文，
// 这里统一转中文，并且所有可恢复错误都附带"另存为 DXF"后备提示。
function friendlyMessage(status: number, fallback: string): string {
  switch (status) {
    case 413:
      return "DWG 文件超过 50MB 上限"
    case 415:
      return "文件不是有效的 DWG"
    case 422:
      return "DWG 无法转换（可能损坏、加密或使用了不支持的特性）。建议在 CAD 里另存为 DXF 后重试。"
    case 504:
      return "转换超时，文件可能过于复杂。建议另存为 DXF。"
    case 502:
    case 503:
    case 500:
      return "转换服务不可用，请稍后重试或导出 DXF。"
    default:
      return fallback
  }
}

export async function extractDwgText(file: File | Blob): Promise<string> {
  const form = new FormData()
  const uploadName = file instanceof File ? file.name : "input.dwg"
  form.append("file", file, uploadName)

  let resp: Response
  try {
    resp = await fetch("/api/convert-dwg", { method: "POST", body: form })
  } catch (e) {
    throw new Error(`转换服务不可用，请稍后重试或在 CAD 里导出 DXF：${String(e)}`)
  }

  if (!resp.ok) {
    throw new Error(friendlyMessage(resp.status, `转换失败 (${resp.status})`))
  }

  const dxfText = await resp.text()
  return extractDxfTextFromString(dxfText)
}
```

- [ ] **Step 2: Typecheck**

Run: `bun run typecheck`
Expected: passes.

- [ ] **Step 3: Commit**

```bash
git add src/lib/drawing-analysis/dwg-reader.ts
git commit -m "feat(dwg): add browser-side DWG reader that round-trips via API"
```

---

## Task 18: Wire `DrawingAnalyzer.tsx` to accept `.dwg`

**Files:**
- Modify: `src/components/editor/DrawingAnalyzer.tsx`

- [ ] **Step 1: Read existing file to find insertion points**

Run: `cat src/components/editor/DrawingAnalyzer.tsx | head -80`
Note the import block, the `accept` attribute on the file input, and the file-handling branch.

- [ ] **Step 2: Add `.dwg` import**

Near the existing `extractDxfText` import, add:

```ts
import { extractDwgText } from "@/lib/drawing-analysis/dwg-reader"
```

- [ ] **Step 3: Update `accept` string**

Find the `<input>` with `accept=".pdf,.dxf,..."` and change to:

```tsx
accept=".pdf,.dxf,.dwg,application/pdf,image/vnd.dxf"
```

- [ ] **Step 4: Branch the file-handling logic by extension**

Find the function that currently calls `extractDxfText(file)`. Add a DWG branch:

```tsx
const name = file.name.toLowerCase()
let text: string
if (name.endsWith(".dwg")) {
  setLoadingHint("转换 DWG 中（预计 2~5 秒）...")
  text = await extractDwgText(file)
} else if (name.endsWith(".dxf")) {
  setLoadingHint("解析 DXF...")
  text = await extractDxfText(file)
} else {
  throw new Error("不支持的文件类型")
}
```

*(If `setLoadingHint` does not exist in the component, either add a simple state variable for it or reuse whatever existing loading-label mechanism the component has. The key requirement is that the user sees a distinct message for DWG because of the multi-second roundtrip.)*

- [ ] **Step 5: Typecheck**

Run: `bun run typecheck`
Expected: passes.

- [ ] **Step 6: Commit**

```bash
git add src/components/editor/DrawingAnalyzer.tsx
git commit -m "feat(dwg): accept .dwg in DrawingAnalyzer"
```

---

## Task 19: `docker-compose.yml` — add sidecar service

**Files:**
- Modify: `docker-compose.yml`
- Modify: `.env.example` (create if missing)

- [ ] **Step 1: Read current compose file**

Run: `cat docker-compose.yml`
Confirm the `plangen` service definition and networks.

- [ ] **Step 2: Replace `docker-compose.yml` with the updated version**

```yaml
services:
  dwg-converter:
    image: ghcr.io/any-leap/zcad-dwg-service:latest
    container_name: dwg-converter
    restart: unless-stopped
    networks:
      - plangen-internal
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/healthz"]
      interval: 30s
      timeout: 5s
      retries: 3
      start_period: 10s

  plangen:
    build: .
    container_name: plangen
    restart: unless-stopped
    env_file:
      - .env
    environment:
      - NODE_ENV=production
      - HOST=0.0.0.0
      - PORT=3000
      - TZ=Asia/Shanghai
      - PLANGEN_DB_PATH=/app/data/plangen.db
      - PLANGEN_ATTACHMENTS_DIR=/app/data/attachments
      - DWG_CONVERTER_URL=http://dwg-converter:8080
    volumes:
      - ./data:/app/data
    command:
      [
        "sh",
        "-c",
        "bun run --bun src/server/db/migrate-run.ts && bun ./.output/server/index.mjs",
      ]
    depends_on:
      dwg-converter:
        condition: service_healthy
    networks:
      - proxy
      - plangen-internal
    labels:
      - traefik.enable=true
      - traefik.http.routers.plangen.rule=Host(`<plangen-domain>`)
      - traefik.http.routers.plangen.entrypoints=websecure
      - traefik.http.routers.plangen.tls.certresolver=le
      - traefik.http.services.plangen.loadbalancer.server.port=3000

networks:
  proxy:
    external: true
  plangen-internal: {}
```

- [ ] **Step 3: Update `.env.example`**

If `.env.example` exists, append:

```
DWG_CONVERTER_URL=http://dwg-converter:8080
```

If not, create it with that single line (plus any other docs you want).

- [ ] **Step 4: Validate compose file**

Run: `docker compose config > /dev/null`
Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add docker-compose.yml .env.example
git commit -m "chore(compose): add dwg-converter sidecar and internal network"
```

---

## Task 20: End-to-end verification

**Files:** (none modified — verification only)

- [ ] **Step 1: Ensure sidecar image is available**

Run: `docker compose pull dwg-converter`
Expected: pulls `ghcr.io/any-leap/zcad-dwg-service:latest` successfully. If not yet pushed by CI, fall back to building locally:
```bash
cd ~/developer/zcad
docker build -f crates/zcad-dwg-service/Dockerfile \
  --build-arg ODA_DEB_URL="<url>" \
  -t ghcr.io/any-leap/zcad-dwg-service:latest .
```

- [ ] **Step 2: Start the stack**

```bash
cd ~/developer/plangen
docker compose up -d dwg-converter
docker compose ps
```
Expected: `dwg-converter` status shows `(healthy)` within 30 seconds.

- [ ] **Step 3: Direct sidecar smoke test from host**

Because `dwg-converter` is only on the internal network, exec into it:
```bash
docker exec dwg-converter curl -fsS http://localhost:8080/healthz
```
Expected: `ok`.

- [ ] **Step 4: Start plangen**

```bash
docker compose up -d plangen
docker compose logs -f plangen
```
Expected: plangen boots, no errors related to `DWG_CONVERTER_URL`.

- [ ] **Step 5: Manual UI test**

1. Open plangen in a browser (`<plangen-domain>` or local)
2. Navigate to the DrawingAnalyzer page
3. Upload a known-good DWG file (2018 format)
4. Expected: within 5 seconds, the extracted text shows up in the UI
5. Repeat with a 2025-format DWG
6. Repeat with a corrupt DWG — expect friendly 415 error message, not a stack trace

- [ ] **Step 6: Regression test — DXF still works**

Upload a DXF file — expected to behave exactly as before (instant, no network roundtrip).

- [ ] **Step 7: Verify acceptance criteria**

Walk through the 8 acceptance criteria from spec §9:
1. ✅ 2018 DWG → text in 5s
2. ✅ 2025 DWG → success
3. ✅ corrupt DWG → friendly error
4. ✅ `docker compose up -d` to ready ≤60s
5. ✅ sidecar kill doesn't break DXF path
6. ✅ `trait DwgConverter` exists, `NativeConverter` stub exists
7. ✅ zcad README documents sidecar build
8. ✅ plangen README documents compose service (**if not done, add a short section now and commit**)

- [ ] **Step 8: Update plangen README**

If the README has no section about DWG / docker compose changes, add one:

```markdown
## DWG 支持

plangen 通过 sidecar 容器 `dwg-converter`（zcad-dwg-service）读取 DWG 文件。
上传 `.dwg` 时，服务端会转发到 sidecar 转成 DXF 后再走 DXF 分析路径。

- 镜像：`ghcr.io/any-leap/zcad-dwg-service:latest`
- 环境变量：`DWG_CONVERTER_URL`（默认 `http://dwg-converter:8080`）
- 最大文件：50 MB；超时：60 秒
- 失败时会给出友好提示，并建议在 CAD 里导出 DXF 作为后备路径
```

Commit:
```bash
git add README.md
git commit -m "docs: document DWG support in plangen README"
```

- [ ] **Step 9: Final verification — all 8 criteria met**

If yes: Phase 1 done. Ship it.
If any criterion fails: stop, document the failure, and loop back to the relevant task before declaring done.

---

## Done criteria

Phase 1 is complete when:
1. All 20 tasks above are checked off
2. All 8 acceptance criteria from spec §9 verified on a running deployment
3. Both repos (`zcad` and `plangen`) have commits pushed to `main`
4. `ghcr.io/any-leap/zcad-dwg-service:latest` is the image in use by plangen
5. User has uploaded a real DWG in production UI and seen the extracted text

Phase 2 (native rust DWG parser replacing `OdaFcConverter`) will be brainstormed and planned separately.
