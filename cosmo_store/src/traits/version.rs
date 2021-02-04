use crate::types::expected_version::ExpectedVersion;
use anyhow::Result;

pub trait Version<Version> {
    fn next_version(&self, version: &ExpectedVersion<Version>) -> Result<Version>;
}
