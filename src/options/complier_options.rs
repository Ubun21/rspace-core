use crate::EntryItem;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct ComplierOptions {
  pub entries: HashMap<String, EntryItem>,
  pub root: String
}