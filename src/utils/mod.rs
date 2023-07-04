use std::path::{Path, Component};

use sugar_path::PathSugar;

pub fn ext_by_module_uri(root: &str, uri: &str) -> String {
  let path = Path::new(uri);
  let mut relative = Path::new(path).relative(root);
  let ext = relative
    .extension()
    .and_then(|uri| uri.to_str())
    .unwrap_or("")
    .to_string();
  relative.set_extension("");
  let mut name = relative
    .components()
    .filter(|c| matches!(c, Component::Normal(_)))
    .filter_map(|c| c.as_os_str().to_str())
    .fold(String::new(), |mut acc, curr| {
      acc.push_str(curr);
      acc.push_str("_");
      acc
    });
  name.push_str(&ext);
  name  
}