use crate::{ComplierOptions, SourceType, PluginContext, ParseModuleArgs, JobContext, BoxModule};

use std::sync::Arc;
use crate::{Plugin};
use std::collections::HashMap;
use nodejs_resolver::Resolver;
pub struct PluginDriver {
  pub(crate) options: Arc<ComplierOptions>,
  pub plugins: Vec<Box<dyn Plugin>>,
  pub resolver: Arc<Resolver>,
  pub module_parser: HashMap<SourceType, usize>
}

impl PluginDriver {
  pub fn new(
    options: Arc<ComplierOptions>,
    plugins: Vec<Box<dyn Plugin>>,
    resolver: Arc<Resolver>,
  ) -> Self {
    let module = plugins
      .iter()
      .enumerate()
      .filter_map(|(index, plugin)| {
        let registered = plugin.register_parse_module(PluginContext::new())?;
        Some(
          registered
            .into_iter()
            .map(|source_type| (source_type, index))
            .collect::<Vec<_>>(),
        )
      })
      .flatten()
      .collect();
    Self {
      options,
      plugins,
      resolver,
      module_parser: module,
    }
  }

  pub fn parse_module(
    &self,
    args: ParseModuleArgs,
    job_ctx: &mut JobContext,
  ) -> anyhow::Result<BoxModule> {
    let parse_index = self
      .module_parser
      .get(
        job_ctx
          .source_type
          .as_ref()
          .ok_or_else(|| anyhow::format_err!("source type not found"))?,
      )
      .unwrap_or_else(|| {
        panic!("no parser found for source type: {:?}", &job_ctx.source_type)
      });
    
      let module =
        self.plugins[*parse_index].parse_module(PluginContext::with_context(job_ctx), args);
      Ok(module)
  }
}