use std::fmt::Debug;
use crate::{PluginContext, SourceType, JobContext, LoadArgs, ParseModuleArgs, BoxModule, RenderManifestArgs};
pub trait Plugin: Debug + Send + Sync {
  fn register_parse_module(&self, _ctx: PluginContext) -> Option<Vec<SourceType>> {
    None
  }
  fn loader(&self, _ctx: PluginContext<& mut JobContext>, args: LoadArgs) -> Option<String> {
    unreachable!()
  }
  fn parse_module(&self, _ctx: PluginContext<& mut JobContext>, args: ParseModuleArgs) -> BoxModule {
    unreachable!()
  }

  fn render_manifest(&self, _ctx: PluginContext, _args: RenderManifestArgs) -> Vec<Asset> {
    vec![]
  }
}

#[derive(Debug)]
pub enum AssetFilename {
  Static(String),
  Templace(String),
}

#[derive(Debug)]
pub struct Asset {
  pub rendered: String,
  pub filename: AssetFilename,
}
