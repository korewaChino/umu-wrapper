use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{instrument, trace, warn};

pub fn generate_prefix_dir(game_id: &str) -> String {
    format!("~/Games/umu/{}", game_id)
}

pub fn bool_to_umu_bool(b: bool) -> String {
    if b {
        "_1_".to_string()
    } else {
        "_0_".to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Global {
    #[serde(default)]
    pub proton: Option<PathBuf>,
    /// Fallback WINE prefix location to use
    #[serde(default)]
    pub prefix: Option<String>,
    /// Fallback game ID to use
    #[serde(default = "default_game_id")]
    pub game_id: String,
    /// Verb to use when launching the game
    #[serde(default)]
    pub proton_verb: Option<String>,
    /// The store the game is from.
    ///
    /// Used by umu-launcher to look up the game's ID
    #[serde(default)]
    pub store: Option<String>,

    /// The default template to use, if not set in the profile
    #[serde(default)]
    pub default_template: Option<String>,
}

fn default_game_id() -> String {
    "0".to_string()
}

/// An application-specific profile struct
///
/// May be used to override the derived template
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    /// The template to derive the profile from
    /// This is required
    pub template: Option<String>,

    /// Name of the profile to launch
    pub name: String,
    /// The ID of the game to launch
    /// This is used to determine which protonfix
    /// to apply to the game.
    ///
    /// It's also used to determine the prefix location
    /// ```txt
    /// Example: `game_id = "umu-123456" # Steam game`
    /// Example: `game_id = "AppID" # EGS game`
    /// ```
    ///
    /// This field is optional, but recommended.
    ///
    /// If not set, the game will be launched with the derived template's prefix, or the default prefix of `0`
    #[serde(default)]
    pub game_id: Option<String>,

    /// The store the game is from.
    ///
    /// Used by umu-launcher to look up the game's ID
    #[serde(default)]
    pub store: Option<String>,

    /// Path to the Proton installation to use
    ///
    /// If not set, umu-wrapper will use the default Proton installation derived from the Template if set
    #[serde(default)]
    pub proton: Option<PathBuf>,
    #[serde(default)]
    pub proton_verb: Option<String>,
    /// WINE prefix location for profiles to use
    /// If not set, umu-wrapper will use the default prefix derived from the Template if set
    /// or auto-generated based on the game's ID
    #[serde(default)]
    pub prefix: Option<String>,
    /// Path to the game's executable
    /// Required for profiles, as it's used to launch the game!
    #[serde(default)]
    pub exe: PathBuf,

    /// Arguments to pass to the game
    /// This is optional, but recommended for games that require additional arguments
    #[serde(default)]
    pub args: Option<Vec<String>>,

    /// Run games inside Steam Linux Runtime
    /// This is for native Linux games that require the Steam Linux Runtime
    #[serde(default)]
    pub no_proton: Option<bool>,
}

impl Profile {
    // pub fn generate_args(&self) -> (String, Vec<String>) {
    //     // split the first argument and the rest of the arguments
    //     let (first, rest) = match &self.args {
    //         Some(args) => {
    //             let default_first = "".to_string();
    //             let (first, rest) = args.split_first().unwrap_or((&default_first, &[]));
    //             (first.clone(), rest.to_vec())
    //         }
    //         None => ("".to_string(), vec![]),
    //     };

    //     (first, rest)
    // }

    #[instrument]
    pub fn run_profile(&self) -> Result<(), crate::error::Error> {
        let mut envs = vec![("GAMEID", self.game_id.as_ref().unwrap().as_str())];

        if let Some(store) = &self.store {
            envs.push(("STORE", store.as_str()));
        }

        let no_proton = self.no_proton.unwrap_or(false);
        let no_proton_str = bool_to_umu_bool(no_proton);

        if no_proton {
            envs.push(("NO_PROTON", &no_proton_str));
        } else {
            if let Some(proton_verb) = &self.proton_verb {
                envs.push(("PROTON_VERB", proton_verb.as_str()));
            }

            if let Some(proton) = &self.proton {
                envs.push(("PROTONPATH", proton.to_str().unwrap()));
            } else {
                return Err(crate::error::Error::NoProton);
            }

            if let Some(prefix) = &self.prefix {
                envs.push(("WINEPREFIX", prefix.as_str()));
            } else {
                warn!("No prefix set for profile, generating one based on game ID");
                // get game ID
                let game_id = self.game_id.clone().unwrap_or("0".to_string());
                let prefix = generate_prefix_dir(&game_id);
                envs.push(("WINEPREFIX", Box::leak(prefix.into_boxed_str())));
            }
        }

        let mut command = std::process::Command::new("umu-run")
            .envs(envs)
            .arg(self.exe.clone())
            .args(self.args.as_ref().unwrap_or(&vec![]))
            .spawn()?;

        command.wait()?;

        Ok(())
    }
}

/// Main configuration struct for umu-wrapper
///
/// Basically defines the re-usable variables for the application
///
/// # Example
///
/// ```toml
/// [[template]]
/// name = "default"
/// prefix = "~/Games/umu/prefix"
/// proton = "/path/to/proton"
/// store = "steam"
/// [[profile]]
/// template = "default"
/// name = "game"
/// game_id = "umu-123456"
/// store = "steam"
/// exe = "game.exe"
/// args = ["-arg1", "-arg2"]
/// ```
///
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Global configuration for umu-wrapper
    /// This is used to define the default values for the application
    #[serde(default)]
    pub global: Global,
    #[serde(default)]
    pub template: Vec<Template>,
    #[serde(default)]
    pub profile: Vec<Profile>,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, crate::error::Error> {
        let config = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config)?;

        Ok(config)
    }

    // Load additional configuration files from a directory
    pub fn load_dir(&mut self, path: &str) -> Result<(), crate::error::Error> {
        // if path not exists, return early
        if !std::path::Path::new(path).exists() {
            warn!("Path {} does not exist, skipping", path);
            return Ok(());
        }
        let dir = std::fs::read_dir(path)?;

        for entry in dir {
            let entry = entry?;
            let path = entry.path();
            let config = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&config)?;
            trace!("Loaded config from {}", path.display());

            self.template.extend(config.template);
            self.profile.extend(config.profile);
        }

        Ok(())
    }

    #[instrument]
    pub fn resolve_template(&self, name: &str) -> Result<Template, crate::error::Error> {
        let tmpl = self
            .template
            .iter()
            .find(|t| t.name == name)
            .ok_or_else(|| color_eyre::eyre::eyre!("Template {} not found in config", name))?;

        // populate values from global

        let mut tmpl = tmpl.clone();

        // set values using Option::or_else
        tmpl.prefix = tmpl.prefix.or_else(|| self.global.prefix.clone());
        tmpl.proton = tmpl.proton.or_else(|| self.global.proton.clone());
        tmpl.proton_verb = tmpl.proton_verb.or_else(|| self.global.proton_verb.clone());
        tmpl.store = tmpl.store.or_else(|| self.global.store.clone());

        Ok(tmpl)
    }
    #[instrument]
    pub fn resolve_profile(&self, name: &str) -> Result<Profile, crate::error::Error> {
        let prof = self
            .profile
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| color_eyre::eyre::eyre!("Profile {} not found in config", name))?;

        let tmpl_name = match &prof.template {
            Some(template) => template,
            None => self
                .global
                .default_template
                .as_ref()
                .ok_or_else(|| color_eyre::eyre::eyre!("No default template set in config"))?,
        };

        // get profile name
        let tmpl = self.resolve_template(tmpl_name)?;

        let mut prof = prof.clone();

        // populate values from template
        prof.proton = prof.proton.or_else(|| tmpl.proton.clone());
        prof.proton_verb = prof.proton_verb.or_else(|| tmpl.proton_verb.clone());
        prof.store = prof.store.or_else(|| tmpl.store.clone());
        prof.game_id = prof.game_id.or_else(|| Some(self.global.game_id.clone()));
        prof.no_proton = prof.no_proton.or(tmpl.no_proton);
        prof.prefix = prof.prefix.or_else(|| tmpl.prefix.clone()).or_else(|| {
            Some(generate_prefix_dir(
                prof.game_id.as_ref().unwrap_or(&self.global.game_id),
            ))
        });

        Ok(prof)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    /// Name of the template
    pub name: String,
    /// WINE prefix location for profiles to use
    ///
    /// If not set, umu-wrapper will generate a prefix based on the game's ID
    /// or ID of 0 if not provided.
    ///
    /// This is the location where the game's files will be stored
    ///
    /// Example: $HOME/Games/umu/$GAMEID
    /// Or: `~/Games/umu-prefix/` (manually set)
    #[serde(default)]
    pub prefix: Option<String>,

    /// Path to the Proton installation to use
    ///
    /// If the profile does not have a proton path set, it will use this path,
    /// so this may be optional.
    ///
    /// But if both the template and profile does not have a proton path set,
    /// umu-wrapper should fail.
    #[serde(default)]
    pub proton: Option<PathBuf>,

    /// Verb to use when launching the game
    #[serde(default)]
    pub proton_verb: Option<String>,

    /// The store the game is from.
    ///
    /// Used by umu-launcher to look up the game's ID
    ///
    /// This is optional, but recommended.
    ///
    #[serde(default)]
    pub store: Option<String>,

    /// Run games inside Steam Linux Runtime
    /// This is for native Linux games that require the Steam Linux Runtime
    #[serde(default)]
    pub no_proton: Option<bool>,
}
