#[derive(Clone, Debug)]
pub enum ExpectedVersion<Version> {
    Any,
    NoStream,
    Exact(Version),
}
