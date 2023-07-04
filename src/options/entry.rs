#[derive(Debug, Clone)]
pub struct EntryItem {
  pub path: String
}

impl From<String> for EntryItem {
  fn from(path: String) -> Self {
    Self { path }
  }
}