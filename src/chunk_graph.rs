use crate::chunk::Chunk;

#[derive(Debug, Default)]
pub struct ChunkGraph {
  id_to_chunk: hashbrown::HashMap<String, Chunk>,
  split_module_point_uri_to_chunk_id: hashbrown::HashMap<String, String>,
}

impl ChunkGraph {
    pub fn add_chunk(&mut self, chunk: Chunk) {
        self
          .split_module_point_uri_to_chunk_id
          .insert(chunk.entry_uri.clone(), chunk.id.clone());
        self.id_to_chunk.insert(chunk.id.clone(), chunk);
    }

    pub fn chunk_by_id(&self, id: &str) -> Option<&Chunk> {
        self.id_to_chunk.get(id)
    }

    pub fn chunk_by_id_mut(&mut self, id: &str) -> Option<&mut Chunk> {
        self.id_to_chunk.get_mut(id)
    }

    pub fn remove_by_id(&mut self, id: &str) -> Option<Chunk> {
        let chunk = self.id_to_chunk.remove(id)?;
        self
          .split_module_point_uri_to_chunk_id
          .remove(&chunk.entry_uri);
        Some(chunk)
    }

    pub fn chunks(&self) -> impl Iterator<Item = &Chunk> {
        self.id_to_chunk.values()
    }

    pub fn chunk_mut(&mut self) -> impl Iterator<Item = &mut Chunk> {
        self.id_to_chunk.values_mut()
    }

    pub fn chunk_by_spit_module_point(&self, uri: &str) -> Option<&Chunk> {
        let chunk_id = self.split_module_point_uri_to_chunk_id.get(uri)?;
        self.id_to_chunk.get(chunk_id)
    }

    pub fn id_to_chunk(&self) -> &hashbrown::HashMap<String, Chunk> {
        &self.id_to_chunk
    }
}
