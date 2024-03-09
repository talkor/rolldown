use std::{
  collections::HashMap,
  fmt::Debug,
  path::{Path, PathBuf},
};

use derivative::Derivative;
use napi::{
  bindgen_prelude::Either,
  threadsafe_function::{ErrorStrategy, ThreadsafeFunction, UnknownReturnValue},
  Error, Status,
};
use napi_derive::napi;
use rolldown_error::BuildError;
use serde::Deserialize;

use super::plugin::{PluginAdapter, PluginOptions};

#[napi(object)]
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct InputItem {
  pub name: Option<String>,
  pub import: String,
}

impl From<InputItem> for rolldown::InputItem {
  fn from(value: InputItem) -> Self {
    Self { name: value.name, import: value.import }
  }
}

#[napi(object)]
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResolveOptions {
  pub alias: Option<HashMap<String, Vec<String>>>,
  pub alias_fields: Option<Vec<Vec<String>>>,
  pub condition_names: Option<Vec<String>>,
  pub exports_fields: Option<Vec<Vec<String>>>,
  pub extensions: Option<Vec<String>>,
  pub main_fields: Option<Vec<String>>,
  pub main_files: Option<Vec<String>>,
  pub modules: Option<Vec<String>>,
  pub symlinks: Option<bool>,
}

impl From<ResolveOptions> for rolldown_resolver::ResolverOptions {
  fn from(value: ResolveOptions) -> Self {
    Self {
      alias: value.alias.map(|alias| alias.into_iter().collect::<Vec<_>>()),
      alias_fields: value.alias_fields,
      condition_names: value.condition_names,
      exports_fields: value.exports_fields,
      extensions: value.extensions,
      main_fields: value.main_fields,
      main_files: value.main_files,
      modules: value.modules,
      symlinks: value.symlinks,
    }
  }
}

#[napi(object, object_to_js = false)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct InputOptions {
  // Not going to be supported
  // @deprecated Use the "inlineDynamicImports" output option instead.
  // inlineDynamicImports?: boolean;

  // acorn?: Record<string, unknown>;
  // acornInjectPlugins?: (() => unknown)[] | (() => unknown);
  // cache?: false | RollupCache;
  // context?: string;sssssssssss
  // experimentalCacheExpiry?: number;
  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  #[napi(
    ts_type = "undefined | ((source: string, importer: string | undefined, isResolved: boolean) => boolean)"
  )]
  pub external: Option<
    ThreadsafeFunction<
      (String, Option<String>, bool),
      Either<bool, UnknownReturnValue>,
      ErrorStrategy::Fatal,
    >,
  >,
  pub input: Vec<InputItem>,
  // makeAbsoluteExternalsRelative?: boolean | 'ifRelativeSource';
  // /** @deprecated Use the "manualChunks" output option instead. */
  // manualChunks?: ManualChunksOption;
  // maxParallelFileOps?: number;
  // /** @deprecated Use the "maxParallelFileOps" option instead. */
  // maxParallelFileReads?: number;
  // moduleContext?: ((id: string) => string | null | void) | { [id: string]: string };
  // onwarn?: WarningHandlerWithDefault;
  // perf?: boolean;
  pub plugins: Vec<PluginOptions>,
  pub resolve: Option<ResolveOptions>,
  // preserveEntrySignatures?: PreserveEntrySignaturesOption;
  // /** @deprecated Use the "preserveModules" output option instead. */
  // preserveModules?: boolean;
  // pub preserve_symlinks: bool,
  // pub shim_missing_exports: bool,
  // strictDeprecations?: boolean;
  // pub treeshake: Option<bool>,
  // watch?: WatcherOptions | false;

  // extra
  pub cwd: String,
  // pub builtins: BuiltinsOptions,
}

impl InputOptions {
  pub(crate) fn to_rolldown_options(
    self,
  ) -> napi::Result<(rolldown::InputOptions, Vec<rolldown_plugin::BoxPlugin>)> {
    let cwd = Path::new(&self.cwd);
    if cwd == PathBuf::from("/") {
      return Err(Error::new(Status::InvalidArg, "cwd cannot be root directory"));
    }

    let external = if let Some(external_fn) = self.external {
      rolldown::External::Fn(Box::new(move |source, importer, is_resolved| {
        let cb = external_fn.clone();
        Box::pin(async move {
          cb.call_async((source, importer, is_resolved))
            .await
            .map(|v| match v {
              Either::A(v) => {
                println!("v: {:?}", v);
                v
              }
              Either::B(_) => false,
            })
            .map_err(BuildError::from)
        })
      }))
    } else {
      rolldown::External::default()
    };

    Ok((
      rolldown::InputOptions {
        input: self.input.into_iter().map(Into::into).collect::<Vec<_>>(),
        cwd: cwd.to_path_buf(),
        external,
        treeshake: false,
        resolve: self.resolve.map(Into::into),
      },
      self.plugins.into_iter().map(PluginAdapter::new_boxed).collect::<Vec<_>>(),
    ))
  }
}
