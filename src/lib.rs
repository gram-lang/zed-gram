//! Gram for Zed: starts the Gram language server, from the `PATH` if it is
//! there, from npm otherwise, and asks it for no view (Zed has no preview).

use std::{env, fs};

use zed_extension_api::{self as zed, LanguageServerId, Result, serde_json::json};

const SERVER_BINARY: &str = "gram-language-server";
const PACKAGE: &str = "@gram-lang/language-server";
const SERVER_PATH: &str = "node_modules/@gram-lang/language-server/dist/server.cjs";
/// The first server that honors `views: false`: an older one would compute
/// the views of every recipe for a panel Zed does not have.
const MIN_VERSION: (u64, u64, u64) = (1, 6, 0);

/// Whether `version` (`1.6.0`, `1.6.0-next.1`...) is at least `MIN_VERSION`.
/// A pre-release of the minimum itself does not count.
fn meets_minimum(version: &str) -> bool {
    let (core, pre) = match version.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (version, None),
    };
    let mut parts = core.split('.').map(|p| p.parse::<u64>().ok());
    let (Some(Some(major)), Some(Some(minor)), Some(Some(patch)), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    let v = (major, minor, patch);
    v > MIN_VERSION || (v == MIN_VERSION && pre.is_none())
}

fn too_old(version: &str) -> String {
    let (major, minor, patch) = MIN_VERSION;
    format!(
        "Gram needs {PACKAGE} {major}.{minor}.{patch} or later, found {version}. \
         Install a newer one with `npm install -g {PACKAGE}`, or wait for its release."
    )
}

struct GramExtension {
    cached_server_path: Option<String>,
}

impl GramExtension {
    /// The server script installed from npm, at its latest version.
    fn server_path(&mut self, id: &LanguageServerId) -> Result<String> {
        if let Some(path) = &self.cached_server_path
            && fs::metadata(path).is_ok_and(|m| m.is_file())
        {
            return Ok(path.clone());
        }

        zed::set_language_server_installation_status(
            id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let installed = zed::npm_package_installed_version(PACKAGE)?;
        let present = fs::metadata(SERVER_PATH).is_ok_and(|m| m.is_file());

        match zed::npm_package_latest_version(PACKAGE) {
            Ok(latest) => {
                if !meets_minimum(&latest) {
                    return Err(self.fail(id, too_old(&latest)));
                }
                if !present || installed.as_deref() != Some(latest.as_str()) {
                    zed::set_language_server_installation_status(
                        id,
                        &zed::LanguageServerInstallationStatus::Downloading,
                    );
                    let result = zed::npm_install_package(PACKAGE, &latest);
                    let present = fs::metadata(SERVER_PATH).is_ok_and(|m| m.is_file());
                    if let Err(error) = result {
                        // An update that fails keeps the server already there.
                        if !present || !installed.as_deref().is_some_and(meets_minimum) {
                            return Err(self.fail(id, error));
                        }
                    }
                }
            }
            // Offline: the server already installed, if recent enough.
            Err(error) => match installed.as_deref() {
                Some(version) if present && meets_minimum(version) => {}
                Some(version) if present => return Err(self.fail(id, too_old(version))),
                _ => return Err(self.fail(id, error)),
            },
        }

        zed::set_language_server_installation_status(
            id,
            &zed::LanguageServerInstallationStatus::None,
        );
        let path = env::current_dir()
            .map_err(|e| e.to_string())?
            .join(SERVER_PATH)
            .to_string_lossy()
            .into_owned();
        self.cached_server_path = Some(path.clone());
        Ok(path)
    }

    fn fail(&self, id: &LanguageServerId, message: String) -> String {
        zed::set_language_server_installation_status(
            id,
            &zed::LanguageServerInstallationStatus::Failed(message.clone()),
        );
        message
    }
}

impl zed::Extension for GramExtension {
    fn new() -> Self {
        Self {
            cached_server_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // A server of your own (`npm install -g`, or a local build) wins.
        // Its version cannot be read from here: the server has no `--version`.
        if let Some(path) = worktree.which(SERVER_BINARY) {
            return Ok(zed::Command {
                command: path,
                args: vec!["--stdio".to_string()],
                env: Default::default(),
            });
        }
        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: vec![self.server_path(id)?, "--stdio".to_string()],
            env: Default::default(),
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        // No preview panel in Zed: ask the server for no view.
        Ok(Some(json!({ "views": false })))
    }
}

zed::register_extension!(GramExtension);

#[cfg(test)]
mod tests {
    use super::meets_minimum;

    #[test]
    fn minimum_version() {
        for ok in ["1.6.0", "1.6.1", "1.7.0", "2.0.0", "1.7.0-next.1", "10.0.0"] {
            assert!(meets_minimum(ok), "{ok}");
        }
        for old in [
            "1.5.1",
            "1.6.0-next.3",
            "0.9.9",
            "1.6",
            "",
            "v1.6.0",
            "1.6.0.1",
        ] {
            assert!(!meets_minimum(old), "{old}");
        }
    }
}
