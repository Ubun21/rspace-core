use crate::{Compilation, ChunkIdAlgo, ModuleGraph, ext_by_module_uri, ChunkKind, Chunk};
use std::collections::{HashMap, HashSet, VecDeque};

pub fn split_code(compilation: &mut Compilation) {

  let module_graph = &compilation.module_graph;
  let mut chunk_id = ChunkIdGenerator {
    id: 0,
    chunk_id_algo: ChunkIdAlgo::Named,
    module_graph,
    root: compilation.options.root.as_str(),
  };

  let is_enable_code_splitting = true;
  let is_reuse_existing_chunk = true;

  let mut chunk_id_by_entry_module_uri:HashMap<&str, String> = HashMap::new();
  let mut chunk_relation_graph2 = petgraph::graphmap::DiGraphMap::<&str, ()>::new();

  let entries = compilation
    .entries_dependencies()
    .iter()
    .filter_map(|dep| module_graph.module_by_dependency(dep))
    .map(|module| module.uri.as_str())
    .collect::<Vec<_>>();

  let chunk_graph = &mut compilation.chunk_graph;

  for entry in &entries {
    let chunk_id = chunk_id.gen_id(*entry);

    let chunk = Chunk::new(
      chunk_id.clone(),
      entry.to_string(),
      ChunkKind::Entry { name: entry.to_string() },
    );

    chunk_id_by_entry_module_uri.insert(*entry, chunk_id.clone());
    chunk_graph.add_chunk(chunk);
  }

  if is_enable_code_splitting {

  }

  let mut mod_to_chunk_id: HashMap<&str, HashSet<&str>> = Default::default();
  for entry in &entries {
    let chunk_id = &chunk_id_by_entry_module_uri[*entry];
    let mut queue = [*entry].into_iter().collect::<VecDeque<_>>();
    let mut visited = HashSet::new();
    while let Some(module_uri) = queue.pop_front() {
        let module = module_graph
          .module_by_uri(module_uri)
          .unwrap_or_else(|| panic!("module not found: {}", module_uri));
        if !visited.contains(module_uri) {
          visited.insert(module_uri);
          mod_to_chunk_id
            .entry(module_uri)
            .or_default()
            .insert(chunk_id.as_str());
          module
            .depended_modules(module_graph)
            .into_iter()
            .for_each(|dep_module| {
              let dep_module_uri = dep_module.uri.as_str();
              queue.push_back(dep_module_uri);
            });
        } else {
          todo!("circular dependency");
        }
    }
  }

  module_graph.modules().for_each(|each_mod| {
    each_mod
      .depended_modules(module_graph)
      .into_iter()
      .for_each(|dep_mod| {
        if let Some(dep_mod_chunk) = chunk_id_by_entry_module_uri.get(dep_mod.uri.as_str()) {
          mod_to_chunk_id[each_mod.uri.as_str()]
            .iter()
            .filter(|each_chunk_id| *each_chunk_id != dep_mod_chunk)
            .for_each(|each_chunk_id| {
              chunk_relation_graph2.add_edge(*each_chunk_id, dep_mod_chunk.as_str(), ());
            });
        }
      });
    });
  println!("mod_to_chunk_id: {:#?}", mod_to_chunk_id);

  for entry in &entries {
    let mut queue = [*entry].into_iter().collect::<VecDeque<_>>();
    let mut visited = HashSet::new();
    while let Some(module_uri) = queue.pop_front() {
      let module = module_graph
        .module_by_uri(module_uri)
        .unwrap_or_else(|| panic!("no entry found for key {:?}", module_uri));
      if !visited.contains(module_uri) {
        visited.insert(module_uri);

        let belong_to_chunks: &HashSet<&str> = &mod_to_chunk_id[module_uri];
        println!(
          "[module {:?}]: belong to chunks {:?}",
          module_uri, belong_to_chunks
        );
        belong_to_chunks
          .iter()
          .filter(|id_of_chunk_to_place_module| {
            if is_reuse_existing_chunk {
              // We only want to have chunks that have no superiors.
              // If both chunk A and B have the same module, we only want to place module into the uppermost chunk based on the relationship between A and B.
              let has_superior = belong_to_chunks.iter().any(|maybe_superior_chunk| {
                chunk_relation_graph2
                  .contains_edge(*maybe_superior_chunk, **id_of_chunk_to_place_module)
              });
              !has_superior
            } else {
              true
            }
          })
          .for_each(|id_of_chunk_to_place_module| {
            let chunk_to_place_module = chunk_graph
              .chunk_by_id_mut(id_of_chunk_to_place_module)
              .unwrap();
            println!(
              "[module {:?}]: place into chunk {:?}",
              module_uri, id_of_chunk_to_place_module);
            chunk_to_place_module
              .module_uris
              .insert(module_uri.to_string());
          });

        module
          .depended_modules(module_graph)
          .into_iter()
          .for_each(|dep_module| queue.push_back(&dep_module.uri));
        if !is_enable_code_splitting {
        }
      } else {
        // TODO: detect circle import
      }
    }
  }


  if true {
    let empty_chunk_id_to_be_removed = chunk_graph
      .chunks()
      .filter(|chunk| chunk.module_uris.is_empty())
      .map(|chunk| chunk.id.clone())
      .collect::<Vec<_>>();

    empty_chunk_id_to_be_removed.iter().for_each(|chunk_id| {
      chunk_graph.remove_by_id(chunk_id);
    });
  }
}

struct ChunkIdGenerator<'a> {
  id: usize,
  chunk_id_algo: ChunkIdAlgo,
  module_graph: &'a ModuleGraph,
  root: &'a str
}

impl<'a> ChunkIdGenerator<'a> {
  pub fn gen_id(&mut self, module_uri: &str) -> String {
    match self.chunk_id_algo {
      ChunkIdAlgo::Numeric => {
        let id = self.id.to_string();
        self.id += 1;
        id
      },
      ChunkIdAlgo::Named => {
        let module = self.module_graph.module_by_uri(module_uri).unwrap();
        ext_by_module_uri(self.root, module.uri.as_str())
      },
    }
  }
}
