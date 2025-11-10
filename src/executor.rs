use crate::paths::get_install_dir;
use anyhow::{Context, Result, anyhow};
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, ExitStatus};

pub fn execute_binary(binary_path: &Path, args: &[OsString]) -> Result<ExitStatus> {
    ensure_within_install_dir(binary_path)?;

    let mut cmd = Command::new(binary_path);
    cmd.args(args);

    let status = cmd
        .status()
        .with_context(|| format!("failed to execute {}", binary_path.display()))?;

    Ok(status)
}

fn ensure_within_install_dir(binary_path: &Path) -> Result<()> {
    let install_dir = get_install_dir()?;
    ensure_binary_within_dir(binary_path, &install_dir)
}

fn ensure_binary_within_dir(binary_path: &Path, install_dir: &Path) -> Result<()> {
    let install_dir = install_dir
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", install_dir.display()))?;
    let binary = binary_path.canonicalize().with_context(|| {
        format!(
            "failed to canonicalize binary path {}",
            binary_path.display()
        )
    })?;

    if !binary.starts_with(&install_dir) {
        return Err(anyhow!(
            "refusing to execute binary outside install dir: {}",
            binary.display()
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn allows_binaries_inside_install_dir() {
        let temp = tempdir().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let binary_path = bin_dir.join("tool");
        fs::write(&binary_path, b"#!/bin/sh\n").unwrap();

        let result = ensure_binary_within_dir(&binary_path, temp.path());
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_binaries_outside_install_dir() {
        let temp = tempdir().unwrap();
        let outside = tempdir().unwrap();
        let binary_path = outside.path().join("tool");
        fs::write(&binary_path, b"#!/bin/sh\n").unwrap();

        let result = ensure_binary_within_dir(&binary_path, temp.path());
        assert!(result.is_err());
    }
}
