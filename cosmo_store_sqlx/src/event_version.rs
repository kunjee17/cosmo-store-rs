use cosmo_store::{ExpectedVersion, Version};
use anyhow::{Result, bail};

fn validate_version(version: &ExpectedVersion<u64>, next_ver: u64) -> Result<u64> {
    match version {
        ExpectedVersion::Any => {
            Ok(next_ver)
        }
        ExpectedVersion::NoStream => {
            if next_ver > 1_u64 {
                bail!("ESERROR_VERSION_STREAMEXISTS: Stream was expected to be empty, but contains {} events", next_ver - 1_u64)
            } else {
                Ok(next_ver)
            }
        }
        ExpectedVersion::Exact(expected_version) => {
            if next_ver != expected_version.clone() {
                bail!("ESERROR_VERSION_VERSIONNOTMATCH: Stream was expected to have next version {0}, but has {1}", next_ver, expected_version )
            } else {
                Ok(next_ver)
            }
        }
    }
}

#[derive(Debug)]
pub struct EventVersion(u64);

impl Version<u64> for EventVersion {
    fn next_version(&self, version: &ExpectedVersion<u64>) -> Result<u64> {
        validate_version(version, self.0 + 1_u64)
    }
}


#[cfg(test)]
mod tests {
    use crate::EventVersion;
    use cosmo_store::{Version, ExpectedVersion};
    use crate::event_version::EventVersion;

    fn test_version() {
        let version = EventVersion(1_u64);
        assert_eq!(1_u64, version.0);
    }
    fn next_version() {
        let version = EventVersion(1_u64);
        let res = version.next_version(&ExpectedVersion::Any).unwrap();
        assert_eq!(2_u64,  res)
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
