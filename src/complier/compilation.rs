use std::sync::{Arc};
use std::collections::HashMap;
use dashmap::DashSet;
use hashbrown::HashSet;

use crate::{ComplierOptions, EntryItem, ModuleGraph, Dependency, ResolveKind, ChunkGraph, split_chunker::split_code};

#[derive(Debug, Default)]
pub struct Compilation {
  pub options: Arc<ComplierOptions>,
  pub entries: HashMap<String, EntryItem>,
  pub (crate) visited_module_id: Arc<DashSet<String>>,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
}

impl Compilation {
    pub fn new(
        options: Arc<ComplierOptions>,
        entries: HashMap<String, EntryItem>,
        visited_module_id: Arc<DashSet<String>>,
        module_graph: ModuleGraph,
    ) -> Self {
        Self {
            options,
            entries,
            visited_module_id,
            module_graph,
            chunk_graph: Default::default(),
        }
    }

    pub fn add_entry(&mut self, key: String, value: EntryItem) {
        self.entries.insert(key, value);
    }

    pub fn entries_dependencies(&self) -> Vec<Dependency> {
        self.entries
            .iter()
            .map(|(_, detail)| {
                Dependency {
                    importer: None,
                    specifier: detail.path.clone(),
                    kind: ResolveKind::Import,
                }  
            })
            .collect()
    }

    pub fn calc_exec_order(&mut self) {
        let mut stack = self
            .entries_dependencies()
            .iter()
            .filter_map(|dep| self.module_graph.module_by_dependency(dep))
            .map(|module| module.uri.clone())
            .collect::<Vec<_>>();
        let mut visited: HashSet<String> = HashSet::new();
        let mut next_exce_order = 0;
        while let Some(uri) = stack.pop() {
            let module = self.module_graph.module_by_uri(&uri).unwrap();
            if !visited.contains(&uri) {
                visited.insert(uri.clone());
                module
                    .depended_modules(&self.module_graph)
                    .into_iter()
                    .rev()
                    .for_each(|dep| {
                        stack.push(dep.uri.clone());
                    });
                self
                    .module_graph
                    .module_by_uri_mut(&uri)
                    .unwrap()
                    .exec_order = next_exce_order;
                next_exce_order += 1;
            }
        }

        let mut modules = self.module_graph.modules().collect::<Vec<_>>();
        modules.sort_by_key(|module| module.exec_order);
    }

    pub fn seal(&mut self) {
        split_code(self);
    }
}