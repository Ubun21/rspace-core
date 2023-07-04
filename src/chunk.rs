use std::collections::HashSet;

use crate::{ModuleGraph, ModuleGraphModule};

#[derive(Debug)]
pub struct Chunk {
  pub id: String,
  pub(crate) entry_uri: String,
  pub(crate) module_uris: HashSet<String>,
  kind: ChunkKind,
}

impl Chunk {
  pub fn new(id: String, entry_uri: String, kind: ChunkKind) -> Self {
    Self {
      id,
      entry_uri,
      module_uris: Default::default(),
      kind,
    }
  }

  pub fn ordered_module<'a>(&self, module_graph: &'a ModuleGraph) -> Vec<&'a ModuleGraphModule> {
    let mut order = self
      .module_uris
      .iter()
      .filter_map(|uri| module_graph.module_by_uri(uri))
      .collect::<Vec<_>>();
    order.sort_by_key(|m| m.exec_order);
    order
  } 
}

#[derive(Debug)]
pub enum ChunkKind {
  Entry { name: String},
  Normal
}

impl ChunkKind {
  pub fn is_entry(&self) -> bool {
    matches!(self, ChunkKind::Entry { .. })
  }

  pub fn is_normal(&self) -> bool {
    matches!(self, ChunkKind::Normal)
  }
}
