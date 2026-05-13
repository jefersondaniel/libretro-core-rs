//! Core option builders and retained frontend registration storage.
//!
//! `CoreOptions` is the v2-first API. It can emit v2, v1, and legacy variable
//! tables while preserving string storage for frontend-retained pointers.

use crate::raw;
use crate::sanitize_cstring;
use std::ffi::{CString, c_char};
use std::ptr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CoreOptionsVersion(u32);

impl CoreOptionsVersion {
    pub const LEGACY_VARIABLES: Self = Self(0);
    pub const V1: Self = Self(1);
    pub const V2: Self = Self(2);

    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn get(self) -> u32 {
        self.0
    }

    pub fn supports_v1(self) -> bool {
        self.0 >= 1
    }

    pub fn supports_v2(self) -> bool {
        self.0 >= 2
    }
}

#[derive(Clone, Debug)]
pub struct VariableDefinition {
    pub key: String,
    pub value: String,
}

impl VariableDefinition {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreOptionValue {
    pub value: String,
    pub label: Option<String>,
}

impl CoreOptionValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: None,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreOptionDefinition {
    pub key: String,
    pub description: String,
    pub description_categorized: Option<String>,
    pub info: Option<String>,
    pub info_categorized: Option<String>,
    pub category_key: Option<String>,
    pub values: Vec<CoreOptionValue>,
    pub default_value: String,
}

impl CoreOptionDefinition {
    pub fn new(
        key: impl Into<String>,
        description: impl Into<String>,
        default_value: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            description_categorized: None,
            info: None,
            info_categorized: None,
            category_key: None,
            values: Vec::new(),
            default_value: default_value.into(),
        }
    }

    pub fn with_value(mut self, value: CoreOptionValue) -> Self {
        self.values.push(value);
        self
    }

    pub fn with_values(mut self, values: impl IntoIterator<Item = CoreOptionValue>) -> Self {
        self.values.extend(values);
        self
    }

    pub fn with_info(mut self, info: impl Into<String>) -> Self {
        self.info = Some(info.into());
        self
    }

    pub fn with_categorized_description(mut self, description: impl Into<String>) -> Self {
        self.description_categorized = Some(description.into());
        self
    }

    pub fn with_categorized_info(mut self, info: impl Into<String>) -> Self {
        self.info_categorized = Some(info.into());
        self
    }

    pub fn with_category(mut self, category_key: impl Into<String>) -> Self {
        self.category_key = Some(category_key.into());
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreOptionCategory {
    pub key: String,
    pub description: String,
    pub info: Option<String>,
}

impl CoreOptionCategory {
    pub fn new(key: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            description: description.into(),
            info: None,
        }
    }

    pub fn with_info(mut self, info: impl Into<String>) -> Self {
        self.info = Some(info.into());
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoreOptions {
    pub categories: Vec<CoreOptionCategory>,
    pub definitions: Vec<CoreOptionDefinition>,
}

impl CoreOptions {
    pub fn new(definitions: impl IntoIterator<Item = CoreOptionDefinition>) -> Self {
        Self {
            categories: Vec::new(),
            definitions: definitions.into_iter().collect(),
        }
    }

    pub fn with_categories(
        mut self,
        categories: impl IntoIterator<Item = CoreOptionCategory>,
    ) -> Self {
        self.categories.extend(categories);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreOptionDisplay {
    pub key: String,
    pub visible: bool,
}

impl CoreOptionDisplay {
    pub fn new(key: impl Into<String>, visible: bool) -> Self {
        Self {
            key: key.into(),
            visible,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreOptionsBuildError {
    TooManyValues { max: usize },
}

#[derive(Default)]
struct StringPool {
    strings: Vec<CString>,
}

impl StringPool {
    fn push(&mut self, value: impl AsRef<str>) -> *const c_char {
        self.strings.push(sanitize_cstring(value.as_ref()));
        self.strings
            .last()
            .expect("just pushed option string")
            .as_ptr()
    }

    fn push_optional(&mut self, value: Option<&str>) -> *const c_char {
        value.map_or(ptr::null(), |value| self.push(value))
    }
}

#[derive(Default)]
pub(crate) struct CoreOptionsStorage {
    strings: StringPool,
    variables: Vec<raw::retro_variable>,
    v1_definitions: Vec<raw::retro_core_option_definition>,
    local_v1_definitions: Vec<raw::retro_core_option_definition>,
    v2_categories: Vec<raw::retro_core_option_v2_category>,
    v2_definitions: Vec<raw::retro_core_option_v2_definition>,
    local_v2_categories: Vec<raw::retro_core_option_v2_category>,
    local_v2_definitions: Vec<raw::retro_core_option_v2_definition>,
}

impl CoreOptionsStorage {
    pub(crate) fn variables(variables: &[VariableDefinition]) -> Self {
        let mut storage = Self::default();
        for variable in variables {
            let key = storage.strings.push(&variable.key);
            let value = storage.strings.push(&variable.value);
            storage.variables.push(raw::retro_variable { key, value });
        }
        storage.variables.push(raw::retro_variable::default());
        storage
    }

    pub(crate) fn legacy_from_options(
        options: &CoreOptions,
    ) -> Result<Self, CoreOptionsBuildError> {
        let mut storage = Self::default();
        for definition in &options.definitions {
            ensure_values_fit(definition)?;
            let key = storage.strings.push(&definition.key);
            let value = storage.strings.push(legacy_definition_value(definition));
            storage.variables.push(raw::retro_variable { key, value });
        }
        storage.variables.push(raw::retro_variable::default());
        Ok(storage)
    }

    pub(crate) fn v1(definitions: &[CoreOptionDefinition]) -> Result<Self, CoreOptionsBuildError> {
        let mut storage = Self::default();
        storage.v1_definitions = storage.raw_v1_definitions(definitions)?;
        storage
            .v1_definitions
            .push(raw::retro_core_option_definition::default());
        Ok(storage)
    }

    pub(crate) fn v1_intl(
        us: &[CoreOptionDefinition],
        local: Option<&[CoreOptionDefinition]>,
    ) -> Result<Self, CoreOptionsBuildError> {
        let mut storage = Self::default();
        storage.v1_definitions = storage.raw_v1_definitions(us)?;
        storage
            .v1_definitions
            .push(raw::retro_core_option_definition::default());
        if let Some(local) = local {
            storage.local_v1_definitions = storage.raw_v1_definitions(local)?;
            storage
                .local_v1_definitions
                .push(raw::retro_core_option_definition::default());
        }
        Ok(storage)
    }

    pub(crate) fn v2(options: &CoreOptions) -> Result<Self, CoreOptionsBuildError> {
        let mut storage = Self::default();
        storage.v2_categories = storage.raw_v2_categories(&options.categories);
        storage
            .v2_categories
            .push(raw::retro_core_option_v2_category::default());
        storage.v2_definitions = storage.raw_v2_definitions(&options.definitions)?;
        storage
            .v2_definitions
            .push(raw::retro_core_option_v2_definition::default());
        Ok(storage)
    }

    pub(crate) fn v2_intl(
        us: &CoreOptions,
        local: Option<&CoreOptions>,
    ) -> Result<Self, CoreOptionsBuildError> {
        let mut storage = Self::v2(us)?;
        if let Some(local) = local {
            storage.local_v2_categories = storage.raw_v2_categories(&local.categories);
            storage
                .local_v2_categories
                .push(raw::retro_core_option_v2_category::default());
            storage.local_v2_definitions = storage.raw_v2_definitions(&local.definitions)?;
            storage
                .local_v2_definitions
                .push(raw::retro_core_option_v2_definition::default());
        }
        Ok(storage)
    }

    pub(crate) fn variables_ptr(&mut self) -> *mut raw::retro_variable {
        self.variables.as_mut_ptr()
    }

    pub(crate) fn v1_definitions_ptr(&mut self) -> *mut raw::retro_core_option_definition {
        self.v1_definitions.as_mut_ptr()
    }

    pub(crate) fn v1_intl_raw(&mut self) -> raw::retro_core_options_intl {
        raw::retro_core_options_intl {
            us: self.v1_definitions.as_mut_ptr(),
            local: if self.local_v1_definitions.is_empty() {
                ptr::null_mut()
            } else {
                self.local_v1_definitions.as_mut_ptr()
            },
        }
    }

    pub(crate) fn v2_raw(&mut self) -> raw::retro_core_options_v2 {
        raw::retro_core_options_v2 {
            categories: if self.v2_categories.is_empty() {
                ptr::null_mut()
            } else {
                self.v2_categories.as_mut_ptr()
            },
            definitions: self.v2_definitions.as_mut_ptr(),
        }
    }

    pub(crate) fn local_v2_raw(&mut self) -> Option<raw::retro_core_options_v2> {
        (!self.local_v2_definitions.is_empty()).then(|| raw::retro_core_options_v2 {
            categories: if self.local_v2_categories.is_empty() {
                ptr::null_mut()
            } else {
                self.local_v2_categories.as_mut_ptr()
            },
            definitions: self.local_v2_definitions.as_mut_ptr(),
        })
    }

    fn raw_v1_definitions(
        &mut self,
        definitions: &[CoreOptionDefinition],
    ) -> Result<Vec<raw::retro_core_option_definition>, CoreOptionsBuildError> {
        definitions
            .iter()
            .map(|definition| self.raw_v1_definition(definition))
            .collect()
    }

    fn raw_v1_definition(
        &mut self,
        definition: &CoreOptionDefinition,
    ) -> Result<raw::retro_core_option_definition, CoreOptionsBuildError> {
        ensure_values_fit(definition)?;
        Ok(raw::retro_core_option_definition {
            key: self.strings.push(&definition.key),
            desc: self.strings.push(&definition.description),
            info: self.strings.push_optional(definition.info.as_deref()),
            values: self.raw_values(definition),
            default_value: self.strings.push(&definition.default_value),
        })
    }

    fn raw_v2_categories(
        &mut self,
        categories: &[CoreOptionCategory],
    ) -> Vec<raw::retro_core_option_v2_category> {
        categories
            .iter()
            .map(|category| raw::retro_core_option_v2_category {
                key: self.strings.push(&category.key),
                desc: self.strings.push(&category.description),
                info: self.strings.push_optional(category.info.as_deref()),
            })
            .collect()
    }

    fn raw_v2_definitions(
        &mut self,
        definitions: &[CoreOptionDefinition],
    ) -> Result<Vec<raw::retro_core_option_v2_definition>, CoreOptionsBuildError> {
        definitions
            .iter()
            .map(|definition| self.raw_v2_definition(definition))
            .collect()
    }

    fn raw_v2_definition(
        &mut self,
        definition: &CoreOptionDefinition,
    ) -> Result<raw::retro_core_option_v2_definition, CoreOptionsBuildError> {
        ensure_values_fit(definition)?;
        Ok(raw::retro_core_option_v2_definition {
            key: self.strings.push(&definition.key),
            desc: self.strings.push(&definition.description),
            desc_categorized: self
                .strings
                .push_optional(definition.description_categorized.as_deref()),
            info: self.strings.push_optional(definition.info.as_deref()),
            info_categorized: self
                .strings
                .push_optional(definition.info_categorized.as_deref()),
            category_key: self
                .strings
                .push_optional(definition.category_key.as_deref()),
            values: self.raw_values(definition),
            default_value: self.strings.push(&definition.default_value),
        })
    }

    fn raw_values(
        &mut self,
        definition: &CoreOptionDefinition,
    ) -> [raw::retro_core_option_value; raw::RETRO_NUM_CORE_OPTION_VALUES_MAX] {
        let mut values =
            [raw::retro_core_option_value::default(); raw::RETRO_NUM_CORE_OPTION_VALUES_MAX];
        for (slot, value) in values.iter_mut().zip(&definition.values) {
            *slot = raw::retro_core_option_value {
                value: self.strings.push(&value.value),
                label: self.strings.push_optional(value.label.as_deref()),
            };
        }
        values
    }
}

fn ensure_values_fit(definition: &CoreOptionDefinition) -> Result<(), CoreOptionsBuildError> {
    if definition.values.len() > raw::RETRO_NUM_CORE_OPTION_VALUES_MAX {
        return Err(CoreOptionsBuildError::TooManyValues {
            max: raw::RETRO_NUM_CORE_OPTION_VALUES_MAX,
        });
    }
    Ok(())
}

fn legacy_definition_value(definition: &CoreOptionDefinition) -> String {
    let mut values = Vec::with_capacity(definition.values.len());
    if definition
        .values
        .iter()
        .any(|value| value.value == definition.default_value)
    {
        values.push(definition.default_value.clone());
        values.extend(
            definition
                .values
                .iter()
                .filter(|value| value.value != definition.default_value)
                .map(|value| value.value.clone()),
        );
    } else {
        values.extend(definition.values.iter().map(|value| value.value.clone()));
    }
    format!("{}; {}", definition.description, values.join("|"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_options_put_default_value_first() {
        let options =
            CoreOptions::new([CoreOptionDefinition::new("demo_speed", "Speed", "normal")
                .with_values([
                    CoreOptionValue::new("slow"),
                    CoreOptionValue::new("normal"),
                    CoreOptionValue::new("fast"),
                ])]);
        let mut storage = CoreOptionsStorage::legacy_from_options(&options).expect("valid options");
        let raw = storage.variables_ptr();

        let value = unsafe { std::ffi::CStr::from_ptr((*raw).value) };
        assert_eq!(value.to_string_lossy(), "Speed; normal|slow|fast");
    }

    #[test]
    fn v2_options_retain_categories_and_labels() {
        let options = CoreOptions::new([CoreOptionDefinition::new(
            "demo_renderer",
            "Renderer",
            "gl",
        )
        .with_category("video")
        .with_categorized_description("API")
        .with_info("Selects the renderer")
        .with_categorized_info("Rendering API")
        .with_values([
            CoreOptionValue::new("gl").with_label("OpenGL"),
            CoreOptionValue::new("soft").with_label("Software"),
        ])])
        .with_categories([CoreOptionCategory::new("video", "Video").with_info("Video settings")]);
        let mut storage = CoreOptionsStorage::v2(&options).expect("valid options");
        let raw = storage.v2_raw();

        let category = unsafe { &*raw.categories };
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(category.desc) }.to_string_lossy(),
            "Video"
        );
        let definition = unsafe { &*raw.definitions };
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(definition.values[0].label) }.to_string_lossy(),
            "OpenGL"
        );
        assert_eq!(
            unsafe { std::ffi::CStr::from_ptr(definition.category_key) }.to_string_lossy(),
            "video"
        );
    }
}
