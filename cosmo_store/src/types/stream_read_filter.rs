#[derive(Clone, Debug)]
pub enum StreamsReadFilter {
    AllStreams,
    StartsWith(String),
    EndsWith(String),
    Contains(String),
}
