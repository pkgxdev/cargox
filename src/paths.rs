use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use std::env;
use std::fs;
use std::path::PathBuf;

pub fn get_install_dir() -> Result<PathBuf> {
    // First check if user has explicitly set an install path
    if let Some(path) = env::var_os("CARGOX_INSTALL_DIR") {
        return Ok(PathBuf::from(path));
    }

    // Use XDG data directory for Linux/Unix or equivalent on other platforms
    if let Some(proj_dirs) = ProjectDirs::from("", "", "cargox") {
        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir)
            .with_context(|| format!("failed to create data directory: {}", data_dir.display()))?;
        return Ok(data_dir.to_path_buf());
    }

    // Fallback to .local/share/cargox
    if let Some(home) = home_dir() {
        let fallback = home.join(".local").join("share").join("cargox");
        fs::create_dir_all(&fallback).with_context(|| {
            format!(
                "failed to create fallback directory: {}",
                fallback.display()
            )
        })?;
        return Ok(fallback);
    }

    Err(anyhow!("unable to determine install directory"))
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::env_lock;

    #[test]
    fn get_install_dir_respects_cargox_install_dir() {
        let _guard = env_lock().lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let custom_path = temp.path().to_path_buf();

        unsafe {
            env::set_var("CARGOX_INSTALL_DIR", &custom_path);
        }
        let result = get_install_dir().unwrap();
        unsafe {
            env::remove_var("CARGOX_INSTALL_DIR");
        }

        assert_eq!(result, custom_path);
    }

    #[test]
    fn get_install_dir_ignores_cargo_install_root() {
        let _guard = env_lock().lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        let cargo_root = temp.path().join("cargo_root");

        // Set CARGO_INSTALL_ROOT - it should be ignored
        unsafe {
            env::set_var("CARGO_INSTALL_ROOT", &cargo_root);
        }
        let result = get_install_dir().unwrap();
        unsafe {
            env::remove_var("CARGO_INSTALL_ROOT");
        }

        // Should NOT use CARGO_INSTALL_ROOT, should use XDG directories instead
        assert_ne!(result, cargo_root);
        assert!(result.to_string_lossy().contains("cargox"));
    }

    #[test]
    fn get_install_dir_uses_xdg_directories() {
        let _guard = env_lock().lock().unwrap();
        // Clear any override env vars
        unsafe {
            env::remove_var("CARGOX_INSTALL_DIR");
        }

        let result = get_install_dir().unwrap();

        // Should contain "cargox" in the path (XDG directory)
        assert!(result.to_string_lossy().contains("cargox"));

        // Should be platform-appropriate
        #[cfg(target_os = "macos")]
        assert!(result.to_string_lossy().contains("Application Support"));

        #[cfg(target_os = "linux")]
        assert!(result.to_string_lossy().contains(".local/share"));

        #[cfg(target_os = "windows")]
        assert!(result.to_string_lossy().contains("AppData"));
    }
}
