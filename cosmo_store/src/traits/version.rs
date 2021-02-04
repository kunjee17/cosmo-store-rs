use anyhow::Result;
use crate::types::expected_version::ExpectedVersion;

pub trait Version<Version> {
    fn next_version(&self, version: &ExpectedVersion<Version>) -> Result<Version>;
}
