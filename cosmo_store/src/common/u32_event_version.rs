use anyhow::{bail, Result};
use crate::types::expected_version::ExpectedVersion;
use crate::traits::version::Version;

fn validate_version(version: &ExpectedVersion<EventVersion>, next_ver: u32) -> Result<u32> {
    match version {
        ExpectedVersion::Any => Ok(next_ver),
        ExpectedVersion::NoStream => {
            if next_ver > 1_u32 {
                bail!("ESERROR_VERSION_STREAMEXISTS: Stream was expected to be empty, but contains {} events", next_ver - 1_u32)
            } else {
                Ok(next_ver)
            }
        }
        ExpectedVersion::Exact(expected_version) => {
            if next_ver != expected_version.0 {
                bail!("ESERROR_VERSION_VERSIONNOTMATCH: Stream was expected to have next version {0}, but has {1}", next_ver, expected_version.0 )
            } else {
                Ok(next_ver)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct EventVersion(pub u32);

impl EventVersion {
    pub fn add(&self, a: u32) -> EventVersion {
        EventVersion(self.0 + a)
    }

    pub fn new(a: u32) -> EventVersion {
        EventVersion(a)
    }
}

impl Version<EventVersion> for EventVersion {
    fn next_version(&self, version: &ExpectedVersion<EventVersion>) -> Result<EventVersion> {
        let res = validate_version(version, self.0 + 1_u32)?;
        Ok(EventVersion(res))
    }
}

#[cfg(test)]
mod tests {
    use crate::common::u32_event_version::EventVersion;
    use crate::traits::version::Version;
    use crate::types::expected_version::ExpectedVersion;

    #[test]
    fn test_version() {
        let version = EventVersion(1_u32);
        assert_eq!(1_u32, version.0);
    }

    #[test]
    fn next_version() {
        let version = EventVersion(1_u32);
        let res = version.next_version(&ExpectedVersion::Any).unwrap();
        assert_eq!(2_u32, res.0)
    }
}
