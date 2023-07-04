use std::fmt::Debug;
use crate::{ModuleDependency, SourceType, Dependency, ModuleGraph, ResolveKind};
#[derive(Debug)]
pub struct ModuleGraphModule {
  pub exec_order: usize,
  pub module: BoxModule,
  pub uri: String,
  pub source_type: SourceType,
  pub dependencies: Vec<Dependency>,
}

impl ModuleGraphModule {
    pub fn new(
      module: BoxModule,
      uri: String,
      source_type: SourceType,
      dependencies: Vec<Dependency>,
    ) -> Self {
      Self {
        exec_order: usize::MAX,
        module,
        uri,
        source_type,
        dependencies,
      }
    }
    pub fn id(&self) -> &str {
      self.uri.as_str()
    }

    pub fn depended_modules<'a>(&self, module_graph: &'a ModuleGraph) -> Vec<&'a ModuleGraphModule> {
      self
        .dependencies
        .iter()
        .filter(|dep| !matches!(dep.kind, ResolveKind::DynamicImport))
        .filter_map(|dep| module_graph.module_by_dependency(dep))
        .collect()
    }
}
pub trait Module: Debug + Send + Sync {
  fn render(&self) -> String;
  fn dependencies(&mut self) -> Vec<ModuleDependency> {
    vec![]
  }
}

pub type BoxModule = Box<dyn Module>;