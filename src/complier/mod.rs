mod compilation;
pub use compilation::*;
use nodejs_resolver::Resolver;

use crate::{Dependency, ModuleGraphModule, ComplierOptions, PluginDriver, Plugin, JobContext, ResolveModuleJob};
use std::sync::{
  atomic::{AtomicUsize, Ordering},
  Arc
};

#[derive(Debug)]
pub enum Msg {
  DependencyReference(Dependency, String),
  TaskFinished(Box<ModuleGraphModule>),
  TaskErrorEncountered(()),
}

pub struct Complier {
  pub options: Arc<ComplierOptions>,
  pub compilation: Compilation,
  pub plugin_driver: Arc<PluginDriver>,
}

impl Complier {
  pub fn new(options: ComplierOptions, plugins: Vec<Box<dyn Plugin>>) -> Self {
    let options = Arc::new(options);
    let plugin_driver = PluginDriver::new(
      options.clone(),
      plugins,
      Arc::new(Resolver::new(nodejs_resolver::ResolverOptions {
        // prefer_relative: false,
        extensions: vec![".tsx", ".jsx", ".ts", ".js", ".json"]
          .into_iter()
          .map(|s| s.to_string())
          .collect(),
        alias_fields: vec![String::from("browser")],
        ..Default::default()
      }))
    );
    Self {
      options,
      compilation: Default::default(),
      plugin_driver: Arc::new(plugin_driver),
    }
  }

  pub async fn compile(&mut self) {
    self.compilation = Compilation::new(
      self.options.clone(),
      self.options.entries.clone(),
      Default::default(),
      Default::default(),
    );

    let active_task_count = Arc::new(AtomicUsize::new(0));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Msg>();

    self.compilation
      .entries_dependencies()
      .into_iter()
      .for_each(|dep| {
        let task = ResolveModuleJob::new(
          JobContext {
            importer: None,
            active_task_acount: active_task_count.clone(),
            visited_module_uri: self.compilation.visited_module_id.clone(),
            source_type: None,
          },
          dep,
          tx.clone(),
          self.plugin_driver.clone(),
        );

        tokio::task::spawn(async move { task.run().await; });
      });

    while active_task_count.load(Ordering::SeqCst) != 0 {
        match rx.recv().await {
          Some(job) => match job {
            Msg::TaskFinished(module) => {
              active_task_count.fetch_sub(1, Ordering::SeqCst);
              self.compilation.module_graph.add_module(*module);
            },
            Msg::DependencyReference(dep, uri) => {
              self.compilation.module_graph.add_dependency(dep, uri);
            },
            Msg::TaskErrorEncountered(()) => {
              active_task_count.fetch_sub(1, Ordering::SeqCst);
            }
          }
          None => {
            tracing::trace!("no more job")
          }
        }
    }
    
    self.compilation.calc_exec_order();

    self.compilation.seal();
  }
}