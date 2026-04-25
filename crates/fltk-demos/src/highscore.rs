//! High-score persistence for `fltk-pong-step4-high-score`.
//!
//! Stores a single `u32` in a text file. Missing/unreadable files are
//! treated as a high score of zero (matches the legacy demo, but without
//! the `unwrap` chain).

use std::io;
use std::path::Path;

/// Read the current high score from `path`. Missing files and malformed
/// payloads both yield `0` so the game can run on a fresh checkout.
///
/// # Errors
///
/// Returns [`io::Error`] for IO errors other than `NotFound`.
pub fn read_high_score(path: &Path) -> io::Result<u32> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(s.trim().parse().unwrap_or(0)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(0),
        Err(e) => Err(e),
    }
}

/// Persist `score` to `path` as a decimal string.
///
/// # Errors
///
/// Returns [`io::Error`] if the file cannot be written.
pub fn write_high_score(path: &Path, score: u32) -> io::Result<()> {
    std::fs::write(path, score.to_string())
}

/// Provable contract: write → read round-trips an arbitrary score.
///
/// # Errors
///
/// Returns [`contracts::ContractError`] if IO fails or the value is not
/// preserved.
pub fn check_highscore_roundtrip(path: &Path) -> Result<(), contracts::ContractError> {
    let target: u32 = 4242;
    let roundtrip = || -> io::Result<u32> {
        write_high_score(path, target)?;
        read_high_score(path)
    };
    let actual = roundtrip().map_err(|e| contracts::ContractError {
        name: "FLTK_HIGHSCORE_ROUNDTRIP",
        message: format!("io error: {e}"),
    })?;
    contracts::check(
        "FLTK_HIGHSCORE_ROUNDTRIP",
        actual == target,
        "write → read must preserve high score",
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use std::env;

    fn tmp_path(name: &str) -> std::path::PathBuf {
        env::temp_dir().join(format!("rgfz-fltk-hs-{}-{}.txt", name, std::process::id()))
    }

    #[test]
    fn missing_file_returns_zero() {
        let path = tmp_path("missing").with_extension("nope");
        let _ = std::fs::remove_file(&path);
        assert_eq!(read_high_score(&path).unwrap(), 0);
    }

    #[test]
    fn write_then_read_roundtrips() {
        let path = tmp_path("roundtrip");
        write_high_score(&path, 1234).unwrap();
        assert_eq!(read_high_score(&path).unwrap(), 1234);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn malformed_file_returns_zero() {
        let path = tmp_path("malformed");
        std::fs::write(&path, "not-a-number\n").unwrap();
        assert_eq!(read_high_score(&path).unwrap(), 0);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn provable_contract_holds() {
        let path = tmp_path("contract");
        check_highscore_roundtrip(&path).expect("contract should hold");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn read_io_error_propagates() {
        let dir = env::temp_dir().join(format!("rgfz-hs-isdir-{}", std::process::id()));
        let _ = std::fs::create_dir(&dir);
        let err = read_high_score(&dir).expect_err("expected non-NotFound IO error");
        assert_ne!(err.kind(), io::ErrorKind::NotFound);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn write_io_error_propagates_through_contract() {
        let bad = env::temp_dir()
            .join(format!("rgfz-hs-no-such-{}", std::process::id()))
            .join("file.txt");
        let err = check_highscore_roundtrip(&bad).expect_err("write should fail");
        assert_eq!(err.name, "FLTK_HIGHSCORE_ROUNDTRIP");
        assert!(err.message.starts_with("io error:"));
    }
}
