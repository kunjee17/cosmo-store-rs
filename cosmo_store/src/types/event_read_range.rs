#[derive(Clone, Debug)]
pub enum EventsReadRange<Version> {
    AllEvents,
    FromVersion(Version),
    ToVersion(Version),
    VersionRange {
        from_version: Version,
        to_version: Version,
    },
}
