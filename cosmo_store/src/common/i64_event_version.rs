use crate::traits::version::Version;
use crate::types::event_read::EventRead;
use crate::types::event_stream::EventStream;
use crate::types::event_write::EventWrite;
use crate::types::expected_version::ExpectedVersion;
use anyhow::{bail, Result};
use chrono::Utc;

fn validate_version(version: &ExpectedVersion<EventVersion>, next_ver: i64) -> Result<i64> {
    match version {
        ExpectedVersion::Any => Ok(next_ver),
        ExpectedVersion::NoStream => {
            if next_ver > 1_i64 {
                bail!("ESERROR_VERSION_STREAMEXISTS: Stream was expected to be empty, but contains {} events", next_ver - 1_i64)
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
pub struct EventVersion(pub i64);

impl EventVersion {
    pub fn add(&self, a: i64) -> EventVersion {
        EventVersion(self.0 + a)
    }

    pub fn new(a: i64) -> EventVersion {
        EventVersion(a)
    }
}

impl Version<EventVersion> for EventVersion {
    fn next_version(&self, version: &ExpectedVersion<EventVersion>) -> Result<EventVersion> {
        let res = validate_version(version, self.0 + 1_i64)?;
        Ok(EventVersion(res))
    }
}

pub fn event_writes_to_reads<Payload: Clone, Meta: Clone>(
    stream_id: &str,
    next: &EventVersion,
    payload: &[EventWrite<Payload, Meta>],
) -> Vec<EventRead<Payload, Meta, EventVersion>> {
    payload
        .iter()
        .enumerate()
        .map(|(i, e)| EventRead::from_event_write(stream_id, next.add(i as i64), Utc::now(), e))
        .collect()
}

pub fn updated_stream(
    stream_id: &str,
    v: i64,
    last: (EventVersion, Option<EventStream<EventVersion>>),
) -> EventStream<EventVersion> {
    match last.1 {
        Some(r) => EventStream {
            last_version: r.last_version.add(v),
            last_updated_utc: Utc::now(),
            ..(r)
        },
        None => EventStream {
            id: stream_id.to_string(),
            last_version: EventVersion::new(v),
            last_updated_utc: Utc::now(),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::common::i64_event_version::EventVersion;
    use crate::traits::version::Version;
    use crate::types::expected_version::ExpectedVersion;

    #[test]
    fn test_version() {
        let version = EventVersion(1_i64);
        assert_eq!(1_i64, version.0);
    }

    #[test]
    fn next_version() {
        let version = EventVersion(1_i64);
        let res = version.next_version(&ExpectedVersion::Any).unwrap();
        assert_eq!(2_i64, res.0)
    }
}
