/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Local support for Verus's `test_verify_one_file! { ... verus_code! { ... } }`
//! test convention, using the installed verifier and the actual MIME specs.

use std::env;
use std::fmt;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use serde_json::Value;
use tempfile::TempDir;

// Token serialization gives these tests the same calling syntax as Verus's
// harness without depending on its nightly-only source-span proc macro. Rust
// string/character literal tokens are preserved; comments and source spans are
// not. Diagnostics refer to the generated test.rs retained on a failure.
macro_rules! verus_code {
    ($($code:tt)*) => {
        concat!("::verus_builtin_macros::verus! {\n", stringify!($($code)*), "\n}")
    };
}

macro_rules! test_verify_one_file {
    ($(#[$attr:meta])* $name:ident $code:expr => $expected:pat $(=> $check:expr)?) => {
        $(#[$attr])*
        fn $name() {
            match common::verify_one_file(stringify!($name), $code) {
                $expected => { $($check;)? }
                result => panic!("{}: unexpected verification result: {result:?}", stringify!($name)),
            }
        }
    };
}

const MIME_VERSION: &str = "0.3.17";
const STATE_MACHINE_VERSION: &str = "0.0.0-2026-06-14-0213";
const TIMEOUT: Duration = Duration::from_secs(120);
static VERIFIER: LazyLock<Verifier> = LazyLock::new(Verifier::new);

struct Verifier {
    executable: PathBuf,
    root: PathBuf,
    mime: PathBuf,
    state_machines: PathBuf,
}

pub struct TestErr {
    status: ExitStatus,
    summary: Value,
    errors: Vec<Value>,
    stderr: String,
    directory: PathBuf,
    command: String,
}

impl fmt::Debug for TestErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "status: {}; summary: {}", self.status, self.summary)?;
        writeln!(f, "input and full logs: {}", self.directory.display())?;
        writeln!(f, "rerun: {}", self.command)?;
        for line in self.stderr.lines() {
            match serde_json::from_str::<Value>(line) {
                Ok(diagnostic) => {
                    if let Some(rendered) = diagnostic["rendered"].as_str() {
                        write!(f, "{rendered}")?;
                    }
                },
                Err(_) => writeln!(f, "{line}")?,
            }
        }
        Ok(())
    }
}

pub fn assert_one_fails(err: TestErr) {
    assert!(
        err.status.code() == Some(1)
            && err.summary["errors"] == 1
            && err.summary["encountered-vir-error"] == false
            && err.errors.len() == 1
            && err.errors[0]["message"] == "assertion failed",
        "expected one verification assertion failure: {err:?}"
    );
}

pub fn assert_rust_error(err: TestErr, code: &str) {
    assert!(
        err.status.code() == Some(1)
            && err.summary["errors"] == 0
            && err.errors.len() == 1
            && err.errors[0]["code"]["code"] == code,
        "expected Rust error {code}, not a verification assertion failure: {err:?}"
    );
}

pub fn verify_one_file(name: &str, code: &str) -> Result<(), TestErr> {
    VERIFIER.verify(name, code)
}

fn test_dir(name: &str) -> TempDir {
    tempfile::Builder::new()
        .prefix(&format!("mime-spec-{name}-"))
        .tempdir()
        .expect("create verification test directory")
}

// Use files rather than pipes so verbose diagnostics cannot block the child.
fn run(command: &mut Command, directory: &Path, label: &str) -> Output {
    let stdout = directory.join(format!("{label}.stdout"));
    let stderr = directory.join(format!("{label}.stderr"));
    let mut child = command
        .stdout(File::create(&stdout).unwrap())
        .stderr(File::create(&stderr).unwrap())
        .spawn()
        .unwrap_or_else(|error| panic!("cannot start {command:?}: {error}"));
    let start = Instant::now();
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .expect("wait for verification test subprocess")
        {
            break status;
        }
        if start.elapsed() >= TIMEOUT {
            let _ = child.kill();
            eprintln!("{command:?} exceeded {TIMEOUT:?}");
            break child
                .wait()
                .expect("reap timed-out verification test subprocess");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    Output {
        status,
        stdout: fs::read(stdout).unwrap(),
        stderr: fs::read(stderr).unwrap(),
    }
}

fn successful_json(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "subprocess failed: {}\n{}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    serde_json::from_slice(&output.stdout).expect("subprocess JSON output")
}

fn find_verus() -> PathBuf {
    let name = env::var_os("VERUS_PATH").unwrap_or_else(|| "verus".into());
    let path = PathBuf::from(&name);
    if path.components().count() > 1 {
        return fs::canonicalize(&path)
            .unwrap_or_else(|error| panic!("invalid VERUS_PATH {path:?}: {error}"));
    }
    for directory in env::split_paths(&env::var_os("PATH").unwrap_or_default()) {
        let candidate = directory
            .join(&name)
            .with_extension(env::consts::EXE_EXTENSION);
        if candidate.is_file() {
            return fs::canonicalize(candidate).expect("resolve Verus executable");
        }
    }
    panic!("Verus not found: set VERUS_PATH to the verus executable or add it to PATH");
}

fn dylib(name: &str) -> String {
    format!(
        "{}{name}{}",
        env::consts::DLL_PREFIX,
        env::consts::DLL_SUFFIX
    )
}

impl Verifier {
    fn new() -> Self {
        let executable = find_verus();
        let root = executable.parent().unwrap().to_path_buf();
        let directory = test_dir("dependencies");
        let version = successful_json(&run(
            Command::new(&executable).args(["--version", "--output-json"]),
            directory.path(),
            "version",
        ));
        let toolchain = version["verus"]["toolchain"]
            .as_str()
            .expect("Verus must report its Rust toolchain");
        eprintln!("MIME specification verifier: {executable:?}\n{version}");

        let net = Path::new(env!("CARGO_MANIFEST_DIR"));
        let lock = fs::read_to_string(net.join("../../../Cargo.lock")).unwrap();
        for (name, version) in [
            ("mime", MIME_VERSION),
            ("verus_state_machines_macros", STATE_MACHINE_VERSION),
        ] {
            assert!(
                lock.contains(&format!("name = \"{name}\"\nversion = \"{version}\"")),
                "review MIME regression expectations for the new {name} version"
            );
        }
        // Build dependencies for Verus's rustc, independently of the native
        // cargo test toolchain. Cargo prunes only this temporary lockfile copy.
        fs::write(directory.path().join("Cargo.lock"), lock).unwrap();
        fs::write(
            directory.path().join("Cargo.toml"),
            format!(
                r#"[package]
name = "mime-spec-test-deps"
version = "0.0.0"
edition = "2021"
[workspace]
[lib]
path = "lib.rs"
[dependencies]
mime = "={MIME_VERSION}"
verus_state_machines_macros = "={STATE_MACHINE_VERSION}"
"#,
            ),
        )
        .unwrap();
        fs::write(directory.path().join("lib.rs"), "").unwrap();
        let test_exe = env::current_exe().unwrap();
        let target = test_exe
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("mime-spec-deps")
            .join(toolchain);
        let build = run(
            Command::new("rustup")
                .args([
                    "run",
                    toolchain,
                    "cargo",
                    "build",
                    "--offline",
                    "--message-format=json",
                ])
                .arg("--target-dir")
                .arg(&target)
                .current_dir(directory.path()),
            directory.path(),
            "dependencies",
        );
        assert!(
            build.status.success(),
            "dependency build failed:\n{}\n{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        let artifacts: Vec<Value> = String::from_utf8_lossy(&build.stdout)
            .lines()
            .map(|line| serde_json::from_str(line).expect("Cargo artifact JSON"))
            .collect();
        let artifact = |name: &str, extension: &str| -> PathBuf {
            artifacts
                .iter()
                .filter(|item| {
                    item["reason"] == "compiler-artifact" && item["target"]["name"] == name
                })
                .flat_map(|item| item["filenames"].as_array().unwrap())
                .filter_map(Value::as_str)
                .map(PathBuf::from)
                .find(|path| path.extension().is_some_and(|ext| ext == extension))
                .unwrap_or_else(|| panic!("Cargo did not produce {name}.{extension}"))
        };
        Self {
            executable,
            root,
            mime: artifact("mime", "rlib"),
            state_machines: artifact("verus_state_machines_macros", env::consts::DLL_EXTENSION),
        }
    }

    fn verify(&self, name: &str, code: &str) -> Result<(), TestErr> {
        let directory = test_dir(name);
        let models =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("mime_classifier_specs/mime_api/mod.rs");
        let entry = directory.path().join("test.rs");
        fs::write(
            &entry,
            format!(
                "#![allow(unused_imports, non_snake_case)]\n\
             #![verifier::auto_reveal_strlit]\n\
             #[path = {models:?}] mod mime_api;\n{code}\n"
            ),
        )
        .unwrap();
        let mut command = Command::new(&self.executable);
        // Verus's own tests use this mode for explicit library selection. It
        // prevents a second bundled state-machine macro from being injected.
        command.args(["--internal-test-mode", "--crate-type=lib", "--edition=2021"]);
        for (name, library) in [
            ("mime", self.mime.clone()),
            ("verus_state_machines_macros", self.state_machines.clone()),
            ("verus_builtin", self.root.join("libverus_builtin.rlib")),
            (
                "verus_builtin_macros",
                self.root.join(dylib("verus_builtin_macros")),
            ),
            ("vstd", self.root.join("libvstd.rlib")),
        ] {
            command
                .arg("--extern")
                .arg(format!("{name}={}", library.display()));
        }
        command
            .arg("--import")
            .arg(format!("vstd={}", self.root.join("vstd.vir").display()));
        for path in [self.root.as_path(), self.mime.parent().unwrap()] {
            command
                .arg("-L")
                .arg(format!("dependency={}", path.display()));
        }
        command
            .args([
                "--verify-root",
                "--output-json",
                "--error-format=json",
                "--multiple-errors",
                "20",
                "--num-threads",
                "1",
            ])
            .arg(&entry)
            .current_dir(directory.path());
        let output = run(&mut command, directory.path(), "verus");
        let report: Value = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
        let summary = report["verification-results"].clone();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        let errors: Vec<Value> = stderr
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter(|diagnostic| {
                diagnostic["level"] == "error"
                    && !diagnostic["message"]
                        .as_str()
                        .unwrap_or_default()
                        .starts_with("aborting due to")
            })
            .collect();
        if output.status.success()
            && errors.is_empty()
            && summary["encountered-error"] == false
            && summary["encountered-vir-error"] == false
            && summary["errors"] == 0
            && summary["verified"].as_u64().is_some_and(|count| count > 0)
        {
            eprintln!("{name}: {summary}");
            Ok(())
        } else {
            Err(TestErr {
                status: output.status,
                summary,
                errors,
                stderr,
                directory: directory.keep(),
                command: format!("{command:?}"),
            })
        }
    }
}
