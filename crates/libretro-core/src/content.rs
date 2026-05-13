//! Content-loading contract helpers shared by `SystemInfo` and `Environment`.
//!
//! `ContentContract` lets a core describe valid extensions, no-game support,
//! fullpath requirements, and persistent data handling once, then apply that
//! same contract to both startup metadata and environment registration.

use crate::{ContentInfoOverride, Environment, SystemInfo};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentContract {
    pub extensions: String,
    pub need_fullpath: bool,
    pub block_extract: bool,
    pub supports_no_game: bool,
    pub persistent_data: bool,
}

impl ContentContract {
    pub fn new(extensions: impl Into<String>) -> Self {
        Self {
            extensions: extensions.into(),
            need_fullpath: false,
            block_extract: false,
            supports_no_game: false,
            persistent_data: false,
        }
    }

    pub fn with_need_fullpath(mut self, need_fullpath: bool) -> Self {
        self.need_fullpath = need_fullpath;
        self
    }

    pub fn with_block_extract(mut self, block_extract: bool) -> Self {
        self.block_extract = block_extract;
        self
    }

    pub fn with_support_no_game(mut self, supports_no_game: bool) -> Self {
        self.supports_no_game = supports_no_game;
        self
    }

    pub fn with_persistent_data(mut self, persistent_data: bool) -> Self {
        self.persistent_data = persistent_data;
        self
    }

    pub fn apply_to_system_info(&self, info: &mut SystemInfo) {
        info.valid_extensions = Some(self.extensions.clone());
        info.need_fullpath = self.need_fullpath;
        info.block_extract = self.block_extract;
    }

    pub fn register_environment(&self, env: &mut Environment<'_>) -> bool {
        let support_ok = if self.supports_no_game {
            env.set_support_no_game(true)
        } else {
            true
        };
        let override_ok = env.set_content_info_overrides(&[self.content_info_override()]);
        support_ok && override_ok
    }

    pub fn content_info_override(&self) -> ContentInfoOverride {
        ContentInfoOverride {
            extensions: self.extensions.clone(),
            need_fullpath: self.need_fullpath,
            persistent_data: self.persistent_data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_contract_applies_system_info_and_override_fields() {
        let contract = ContentContract::new("bin|dat")
            .with_need_fullpath(true)
            .with_block_extract(true)
            .with_support_no_game(false)
            .with_persistent_data(true);
        let mut info = SystemInfo::new("test", "0.1");

        contract.apply_to_system_info(&mut info);
        let override_info = contract.content_info_override();

        assert_eq!(info.valid_extensions.as_deref(), Some("bin|dat"));
        assert!(info.need_fullpath);
        assert!(info.block_extract);
        assert_eq!(override_info.extensions, "bin|dat");
        assert!(override_info.need_fullpath);
        assert!(override_info.persistent_data);
    }
}
