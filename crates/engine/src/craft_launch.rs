//! Starting a sibling Craft app (DeckCraft AR, GridCraft AR) on a `.craftx` file, by the spec's
//! launching rules (`docs/ar-exchange.md`): the executable beside ours, then `..\<Sibling> AR\`
//! under the same parent folder, then (Windows) the `App Paths` registry keys the AR installers
//! write — the AR key first, the upstream one second — then the plain name on `PATH`. When
//! nothing is found the caller keeps the file and tells the user where it is; nothing here fails
//! a command.
//!
//! Tests never start anything: the launcher is off under `cfg(test)` and when the environment
//! has `WORDCRAFT_NO_LAUNCH` set (the same pattern as `WORDCRAFT_SPEECH=off`).

use std::path::{Path, PathBuf};

/// The two apps a document can be sent to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sibling {
    DeckCraft,
    GridCraft,
}

impl Sibling {
    /// The product name shown to the user.
    pub fn name(self) -> &'static str {
        match self {
            Sibling::DeckCraft => "DeckCraft AR",
            Sibling::GridCraft => "GridCraft AR",
        }
    }
    /// The executable's name without extension (also the name looked up on `PATH`).
    pub fn exe_stem(self) -> &'static str {
        match self {
            Sibling::DeckCraft => "deckcraft",
            Sibling::GridCraft => "gridcraft",
        }
    }
    /// The executable's file name on this platform.
    pub fn exe_name(self) -> String {
        if cfg!(windows) { format!("{}.exe", self.exe_stem()) } else { self.exe_stem().to_string() }
    }
    /// The `App Paths` keys the installers register, the AR one first.
    pub fn app_paths_keys(self) -> [String; 2] {
        [format!("{}-ar.exe", self.exe_stem()), format!("{}.exe", self.exe_stem())]
    }
}

/// How a launch went.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Launched {
    /// The sibling started, from this executable.
    Started(PathBuf),
    /// No installation of the sibling was found; nothing was started.
    NotFound,
}

/// Start `app` with `path` as its only argument, trying the spec's locations in order. `Err`
/// only when an executable was found but could not be started.
pub fn launch(app: Sibling, path: &Path) -> Result<Launched, String> {
    if !enabled() {
        return Ok(Launched::NotFound);
    }
    let Some(exe) = find(app) else { return Ok(Launched::NotFound) };
    start(&exe, path)?;
    Ok(Launched::Started(exe))
}

/// Whether launching is allowed in this process (off in tests and with `WORDCRAFT_NO_LAUNCH`).
pub fn enabled() -> bool {
    !cfg!(test) && std::env::var_os("WORDCRAFT_NO_LAUNCH").is_none_or(|v| v.is_empty() || v == "0")
}

/// The sibling's executable, by the spec's rules, if it is installed.
pub fn find(app: Sibling) -> Option<PathBuf> {
    candidates(app).into_iter().find(|p| p.is_file())
}

/// The locations to try, in order, before `PATH`.
fn candidates(app: Sibling) -> Vec<PathBuf> {
    let mut v = Vec::new();
    let exe_name = app.exe_name();
    if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)) {
        // 1. Beside the current executable, then `..\<Sibling> AR\` under the same parent.
        v.push(dir.join(&exe_name));
        if let Some(parent) = dir.parent() {
            v.push(parent.join(app.name()).join(&exe_name));
        }
    }
    // 2. Windows: the App Paths keys.
    if cfg!(windows) {
        for key in app.app_paths_keys() {
            if let Some(p) = app_path(&key) {
                v.push(p);
            }
        }
    }
    // 3. On PATH.
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            if !dir.as_os_str().is_empty() {
                v.push(dir.join(&exe_name));
            }
        }
    }
    v
}

/// The default value of `HKLM\Software\Microsoft\Windows\CurrentVersion\App Paths\<key>` through
/// `reg query` (no registry crate); `None` when the key is missing or the output is odd.
fn app_path(key: &str) -> Option<PathBuf> {
    if !cfg!(windows) {
        return None;
    }
    let full = format!(r"HKLM\Software\Microsoft\Windows\CurrentVersion\App Paths\{key}");
    let out = std::process::Command::new("reg").args(["query", &full, "/ve"]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    parse_reg_default(&String::from_utf8_lossy(&out.stdout))
}

/// The `REG_SZ` value of a `reg query … /ve` listing:
/// `    (Default)    REG_SZ    C:\Program Files\DeckCraft AR\deckcraft.exe`.
pub fn parse_reg_default(listing: &str) -> Option<PathBuf> {
    for line in listing.lines() {
        // The type word (`REG_SZ`, `REG_EXPAND_SZ`); the value is everything after it.
        let Some(at) = line.find("REG_") else { continue };
        let rest = line.get(at..).unwrap_or("");
        let Some((_, value)) = rest.split_once(char::is_whitespace) else { continue };
        let value = value.trim();
        if !value.is_empty() {
            return Some(PathBuf::from(value.trim_matches('"')));
        }
    }
    None
}

fn start(exe: &Path, path: &Path) -> Result<(), String> {
    std::process::Command::new(exe).arg(path).spawn().map(|_| ()).map_err(|e| format!("could not start {}: {e}", exe.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launching_is_off_in_tests_and_never_fails_without_a_sibling() {
        assert!(!enabled());
        assert_eq!(launch(Sibling::DeckCraft, Path::new("x.craftx")), Ok(Launched::NotFound));
        assert_eq!(launch(Sibling::GridCraft, Path::new("x.craftx")), Ok(Launched::NotFound));
    }

    #[test]
    fn names_follow_the_spec() {
        assert_eq!(Sibling::DeckCraft.name(), "DeckCraft AR");
        assert_eq!(Sibling::GridCraft.exe_stem(), "gridcraft");
        assert_eq!(Sibling::DeckCraft.app_paths_keys(), ["deckcraft-ar.exe".to_string(), "deckcraft.exe".to_string()]);
        let c = candidates(Sibling::DeckCraft);
        assert!(c.first().is_some_and(|p| p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("deckcraft"))), "{c:?}");
        assert!(c.get(1).is_some_and(|p| p.to_string_lossy().contains("DeckCraft AR")), "{c:?}");
    }

    #[test]
    fn reg_listings_parse_and_odd_output_is_none() {
        let listing = "\r\nHKEY_LOCAL_MACHINE\\Software\\...\\deckcraft-ar.exe\r\n    (Default)    REG_SZ    C:\\Program Files\\DeckCraft AR\\deckcraft.exe\r\n\r\n";
        assert_eq!(parse_reg_default(listing), Some(PathBuf::from(r"C:\Program Files\DeckCraft AR\deckcraft.exe")));
        assert_eq!(parse_reg_default("    (Default)    REG_EXPAND_SZ    \"C:\\x\\deckcraft.exe\""), Some(PathBuf::from(r"C:\x\deckcraft.exe")));
        assert_eq!(parse_reg_default(""), None);
        assert_eq!(parse_reg_default("ERROR: The system was unable to find the specified registry key or value."), None);
        assert_eq!(parse_reg_default("    (Default)    REG_SZ    "), None);
    }
}
