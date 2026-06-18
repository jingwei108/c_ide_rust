#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct SourceLoc {
    pub line: i32,
    pub column: i32,
}
