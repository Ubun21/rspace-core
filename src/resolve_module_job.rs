use std::{
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  }
};
use dashmap::DashSet;
use crate::{SourceType, PluginDriver, Msg, ResolveArgs, LoadArgs, ParseModuleArgs, ModuleGraphModule};
use tokio::sync::mpsc::UnboundedSender;
use std::path::Path;
use tracing::trace;
use nodejs_resolver::ResolveResult;


#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Dependency {
  /// Uri of importer module
  pub importer: Option<String>,
  /// `./a.js` in `import './a.js'` is specifier
  pub specifier: String,
  pub kind: ResolveKind,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ResolveKind {
  Import,
  Require,
  DynamicImport,
  AtImport,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ModuleDependency {
  pub specifier: String,
  pub kind: ResolveKind,
}
#[derive(Debug, Clone)]
pub struct JobContext {
  pub importer: Option<String>,
  pub(crate) active_task_acount: Arc<AtomicUsize>,
  pub(crate) visited_module_uri: Arc<DashSet<String>>,
  pub source_type: Option<SourceType>,
}

impl JobContext {
  pub fn set_source_type(&mut self, source_type: SourceType) {
    self.source_type = Some(source_type);
  }
}

pub struct ResolveModuleJob {
  pub context: JobContext,
  pub dependency: Dependency,
  pub tx: UnboundedSender<Msg>,
  pub plugin_driver: Arc<PluginDriver>,
}

impl ResolveModuleJob {
  pub fn new(
    context: JobContext,
    dependency: Dependency,
    tx: UnboundedSender<Msg>,
    plugin_driver: Arc<PluginDriver>,
  ) -> Self {
    context.active_task_acount.fetch_add(1, Ordering::SeqCst);

    Self {
      context,
      dependency,
      tx,
      plugin_driver,
    }
  }

  pub async fn run(mut self) {
    let uri = resolve(ResolveArgs { 
      importer: self.dependency.importer.as_deref(), 
      specifier: self.dependency.specifier.as_str(), 
      kind: self.dependency.kind 
    },
    &self.plugin_driver);
    trace!("resolved uri: {:?}", uri);
    let source_type = resolve_source_type_by_uri(&uri);
    self.context.set_source_type(source_type.unwrap()); 
    self
      .tx
      .send(
        Msg::DependencyReference(self.dependency.clone(), 
        uri.clone()
    ))
    .unwrap();
    if self.context.visited_module_uri.contains(&uri) {
      self.tx.send(Msg::TaskErrorEncountered(())).unwrap();
    } else {
      self.context.visited_module_uri.insert(uri.clone());
      let source = load(LoadArgs { uri: uri.as_str() }).await;
      let mut module = self
        .plugin_driver
        .parse_module(
          ParseModuleArgs {
            uri: uri.as_str(),
            source: source,
          },
          &mut self.context,
        )
        .unwrap();

      let deps = module
        .dependencies()
        .iter()
        .map(|dep| Dependency {
          importer: Some(uri.clone()),
          specifier: dep.specifier.clone(),
          kind: dep.kind,
        })
        .collect::<Vec<_>>();

      deps.iter().for_each(|dep| {
        self.fork(dep.clone());
      });

      self
        .tx
        .send(
          Msg::TaskFinished(Box::new(ModuleGraphModule::new(
            module,
            uri,
            source_type.unwrap(),
            deps,
          ))),
        )
        .unwrap();

    }
    println!("running")
  }

  pub fn fork(&self, dep: Dependency) {
    let context = self.context.clone();
    let task = ResolveModuleJob::new(context, dep, self.tx.clone(), self.plugin_driver.clone());
    tokio::task::spawn(async move {
      task.run().await;
    });
  }
}

pub fn resolve_source_type_by_uri<T: AsRef<Path>>(uri: T) -> Option<SourceType> {
  let path = uri.as_ref();
  let ext = path.extension()?.to_str()?;
  let source_type: Option<SourceType> = ext.try_into().ok();
  source_type
}

pub async fn load(args: LoadArgs<'_>) -> String {
  tokio::fs::read_to_string(args.uri)
    .await
    .unwrap_or_else(|_| panic!("fail to load uri: {:?}", args.uri))
}

pub fn resolve(args: ResolveArgs, plugin_driver: &PluginDriver) -> String {
  if let Some(importer) = args.importer {
    let base_dir = Path::new(importer).parent().unwrap();
    tracing::trace!(
      "resolved importer:{:?},specifier:{:?}",
      importer,
      args.specifier
    );
    match plugin_driver
      .resolver
      .resolve(base_dir, args.specifier)
      .unwrap_or_else(|_| {
        panic!(
          "fail to resolved importer:{:?},specifier:{:?}",
          importer, args.specifier
        )
    }) {
      ResolveResult::Path(path) => path.to_string_lossy().to_string(),
      _ => {
        tracing::trace!(
          "resolved importer:{:?},specifier:{:?} to None",
          importer,
          args.specifier
        );
        args.specifier.to_string()
      }
    }
  } else {
    Path::new(plugin_driver.options.root.as_str())
      .join(args.specifier)
      .to_string_lossy()
      .to_string()
  }
}