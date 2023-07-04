mod module;
pub use module::*;
mod module_graph;
mod plugin;
pub use plugin::*;
pub use module_graph::*;
mod resolve_module_job;
pub use resolve_module_job::*;
mod options;
pub use options::*;
mod complier;
pub use complier::*;
mod chunk_spliter;
pub use chunk_spliter::*;
mod chunk;
pub use chunk::*;
mod chunk_graph;
pub use chunk_graph::*;
mod utils;
pub use utils::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceType {
  Json,
  Css,
  Js,
  Jsx,
  Tsx,
  Ts,
}

impl TryFrom<&str> for SourceType {
  type Error = ();

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "json" => Ok(Self::Json),
      "css" => Ok(Self::Css),
      "js" => Ok(Self::Js),
      "jsx" => Ok(Self::Jsx),
      "tsx" => Ok(Self::Tsx),
      "ts" => Ok(Self::Ts),
      _ => Err(()),
    }
  }
}
