use std::{borrow::Cow, collections::HashMap};

use derivative::Derivative;
use futures::TryFutureExt;
use napi::{
  bindgen_prelude::{Either, Either3, Promise},
  threadsafe_function::{ThreadsafeFunction, UnknownReturnValue},
  Error, Status,
};
use rolldown_plugin::Plugin;
use serde::Deserialize;

use crate::{
  options::sourcemap::SourceMap,
  types::{binding_outputs::BindingOutputs, binding_rendered_module::BindingRenderedModule},
};

#[napi_derive::napi(object, object_to_js = false)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct PluginOptions {
  pub name: String,

  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  #[napi(ts_type = "() => Promise<void>")]
  pub build_start: Option<ThreadsafeFunction<(), Either<Promise<()>, UnknownReturnValue>>>,

  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  #[napi(
    ts_type = "(specifier: string, importer?: string, options?: HookResolveIdArgsOptions) => Promise<undefined | ResolveIdResult>"
  )]
  pub resolve_id: Option<
    ThreadsafeFunction<
      (String, Option<String>, Option<HookResolveIdArgsOptions>),
      Either3<Promise<Option<ResolveIdResult>>, Option<ResolveIdResult>, UnknownReturnValue>,
    >,
  >,

  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  #[napi(ts_type = "(id: string) => Promise<undefined | SourceResult>")]
  pub load: Option<
    ThreadsafeFunction<
      String,
      Either3<Promise<Option<SourceResult>>, Option<SourceResult>, UnknownReturnValue>,
    >,
  >,

  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  #[napi(ts_type = "(id: string, code: string) => Promise<undefined | SourceResult>")]
  pub transform: Option<
    ThreadsafeFunction<
      (String, String),
      Either3<Promise<Option<SourceResult>>, Option<SourceResult>, UnknownReturnValue>,
    >,
  >,

  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  #[napi(ts_type = "(error?: string) => Promise<void>")]
  pub build_end:
    Option<ThreadsafeFunction<Option<String>, Either<Promise<()>, UnknownReturnValue>>>,

  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  #[napi(
    ts_type = "(code: string, chunk: RenderedChunk) => Promise<undefined | HookRenderChunkOutput>"
  )]
  pub render_chunk: Option<
    ThreadsafeFunction<
      (String, RenderedChunk),
      Either3<
        Promise<Option<HookRenderChunkOutput>>,
        Option<HookRenderChunkOutput>,
        UnknownReturnValue,
      >,
    >,
  >,

  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  #[napi(ts_type = "(bundle: Outputs, isWrite: boolean) => Promise<void>")]
  pub generate_bundle:
    Option<ThreadsafeFunction<(BindingOutputs, bool), Either<Promise<()>, UnknownReturnValue>>>,

  #[derivative(Debug = "ignore")]
  #[serde(skip_deserializing)]
  #[napi(ts_type = "(bundle: Outputs) => Promise<void>")]
  pub write_bundle:
    Option<ThreadsafeFunction<BindingOutputs, Either<Promise<()>, UnknownReturnValue>>>,
}

#[napi_derive::napi(object)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct HookResolveIdArgsOptions {
  pub is_entry: bool,
  pub kind: String,
}

impl From<rolldown_plugin::HookResolveIdArgsOptions> for HookResolveIdArgsOptions {
  fn from(value: rolldown_plugin::HookResolveIdArgsOptions) -> Self {
    Self { is_entry: value.is_entry, kind: value.kind.to_string() }
  }
}

#[napi_derive::napi(object)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct ResolveIdResult {
  pub id: String,
  pub external: Option<bool>,
}

impl From<ResolveIdResult> for rolldown_plugin::HookResolveIdOutput {
  fn from(value: ResolveIdResult) -> Self {
    Self { id: value.id, external: value.external }
  }
}

#[napi_derive::napi(object)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct SourceResult {
  pub code: String,
  pub map: Option<SourceMap>,
}

impl From<SourceResult> for rolldown_plugin::HookLoadOutput {
  fn from(value: SourceResult) -> Self {
    Self { code: value.code, map: value.map.map(Into::into) }
  }
}

#[napi_derive::napi(object)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derivative(Debug)]
pub struct HookRenderChunkOutput {
  pub code: String,
}

impl From<HookRenderChunkOutput> for rolldown_plugin::HookRenderChunkOutput {
  fn from(value: HookRenderChunkOutput) -> Self {
    Self { code: value.code }
  }
}

#[napi_derive::napi(object)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derive(Debug)]
pub struct PreRenderedChunk {
  // pub name: String,
  pub is_entry: bool,
  pub is_dynamic_entry: bool,
  pub facade_module_id: Option<String>,
  pub module_ids: Vec<String>,
  pub exports: Vec<String>,
}

impl From<rolldown::PreRenderedChunk> for PreRenderedChunk {
  fn from(value: rolldown::PreRenderedChunk) -> Self {
    Self {
      is_entry: value.is_entry,
      is_dynamic_entry: value.is_dynamic_entry,
      facade_module_id: value.facade_module_id,
      module_ids: value.module_ids,
      exports: value.exports,
    }
  }
}

#[napi_derive::napi(object)]
#[derive(Deserialize, Default, Derivative)]
#[serde(rename_all = "camelCase")]
#[derive(Debug)]
pub struct RenderedChunk {
  // PreRenderedChunk
  pub is_entry: bool,
  pub is_dynamic_entry: bool,
  pub facade_module_id: Option<String>,
  pub module_ids: Vec<String>,
  pub exports: Vec<String>,
  // RenderedChunk
  pub file_name: String,
  #[serde(skip)]
  pub modules: HashMap<String, BindingRenderedModule>,
}

impl From<rolldown_common::RenderedChunk> for RenderedChunk {
  fn from(value: rolldown_common::RenderedChunk) -> Self {
    Self {
      is_entry: value.is_entry,
      is_dynamic_entry: value.is_dynamic_entry,
      facade_module_id: value.facade_module_id,
      module_ids: value.module_ids,
      exports: value.exports,
      file_name: value.file_name,
      modules: value.modules.into_iter().map(|(key, value)| (key, value.into())).collect(),
    }
  }
}

impl PluginOptions {
  pub(crate) fn boxed(self) -> Box<dyn Plugin> {
    Box::new(self)
  }
}

#[async_trait::async_trait]
impl Plugin for PluginOptions {
  fn name(&self) -> Cow<'static, str> {
    Cow::Owned(self.name.clone())
  }

  #[allow(clippy::redundant_closure_for_method_calls)]
  async fn build_start(
    &self,
    _ctx: &mut rolldown_plugin::PluginContext,
  ) -> rolldown_plugin::HookNoopReturn {
    if let Some(cb) = &self.build_start {
      cb.call_async(Ok(()))
        .and_then(|start| async {
          match start {
            Either::A(p) => {
              let result = p.await?;
              Ok(result)
            }
            Either::B(_) => Ok(()),
          }
        })
        .await?;
    }
    Ok(())
  }

  #[allow(clippy::redundant_closure_for_method_calls)]
  async fn resolve_id(
    &self,
    _ctx: &mut rolldown_plugin::PluginContext,
    args: &rolldown_plugin::HookResolveIdArgs,
  ) -> rolldown_plugin::HookResolveIdReturn {
    if let Some(cb) = &self.resolve_id {
      let res = cb
        .call_async(Ok((
          args.source.to_string(),
          args.importer.map(|s| s.to_string()),
          Some(args.options.clone().into()),
        )))
        .and_then(|cb| async {
          match cb {
            Either3::A(p) => {
              let result = p.await?;
              Ok(result)
            }
            Either3::B(result) => Ok(result),
            Either3::C(_) => {
              Err(Error::new(Status::InvalidArg, "Invalid return value from resolve_id hook"))
            }
          }
        })
        .await?;

      Ok(res.map(Into::into))
    } else {
      Ok(None)
    }
  }

  #[allow(clippy::redundant_closure_for_method_calls)]
  async fn load(
    &self,
    _ctx: &mut rolldown_plugin::PluginContext,
    args: &rolldown_plugin::HookLoadArgs,
  ) -> rolldown_plugin::HookLoadReturn {
    if let Some(cb) = &self.load {
      let res = cb
        .call_async(Ok(args.id.to_string()))
        .and_then(|loaded| async {
          match loaded {
            Either3::A(p) => {
              let result = p.await?;
              Ok(result)
            }
            Either3::B(result) => Ok(result),
            Either3::C(_) => {
              Err(Error::new(Status::InvalidArg, "Invalid return value from load hook"))
            }
          }
        })
        .await?;
      Ok(res.map(Into::into))
    } else {
      Ok(None)
    }
  }

  #[allow(clippy::redundant_closure_for_method_calls)]
  async fn transform(
    &self,
    _ctx: &mut rolldown_plugin::PluginContext,
    args: &rolldown_plugin::HookTransformArgs,
  ) -> rolldown_plugin::HookTransformReturn {
    if let Some(cb) = &self.transform {
      let res = cb
        .call_async(Ok((args.code.to_string(), args.id.to_string())))
        .and_then(|transformed| async {
          match transformed {
            Either3::A(p) => {
              let result = p.await?;
              Ok(result)
            }
            Either3::B(result) => Ok(result),
            Either3::C(_) => {
              Err(Error::new(Status::InvalidArg, "Invalid return value from transform hook"))
            }
          }
        })
        .await?;
      Ok(res.map(Into::into))
    } else {
      Ok(None)
    }
  }

  #[allow(clippy::redundant_closure_for_method_calls)]
  async fn build_end(
    &self,
    _ctx: &mut rolldown_plugin::PluginContext,
    args: Option<&rolldown_plugin::HookBuildEndArgs>,
  ) -> rolldown_plugin::HookNoopReturn {
    if let Some(cb) = &self.build_end {
      cb.call_async(Ok(args.map(|a| a.error.to_string())))
        .and_then(|build_end| async {
          match build_end {
            Either::A(p) => {
              let result = p.await?;
              Ok(result)
            }
            Either::B(_) => Ok(()),
          }
        })
        .await?;
    }
    Ok(())
  }

  #[allow(clippy::redundant_closure_for_method_calls)]
  async fn render_chunk(
    &self,
    _ctx: &rolldown_plugin::PluginContext,
    args: &rolldown_plugin::RenderChunkArgs,
  ) -> rolldown_plugin::HookRenderChunkReturn {
    if let Some(cb) = &self.render_chunk {
      let res = cb
        .call_async(Ok((args.code.to_string(), args.chunk.clone().into())))
        .and_then(|rendered| async {
          match rendered {
            Either3::A(p) => {
              let result = p.await?;
              Ok(result)
            }
            Either3::B(result) => Ok(result),
            Either3::C(_) => {
              Err(Error::new(Status::InvalidArg, "Invalid return value from render_chunk hook"))
            }
          }
        })
        .await?;
      return Ok(res.map(Into::into));
    }
    Ok(None)
  }

  #[allow(clippy::redundant_closure_for_method_calls)]
  async fn generate_bundle(
    &self,
    _ctx: &rolldown_plugin::PluginContext,
    bundle: &Vec<rolldown_common::Output>,
    is_write: bool,
  ) -> rolldown_plugin::HookNoopReturn {
    if let Some(cb) = &self.generate_bundle {
      cb.call_async(Ok((bundle.clone().into(), is_write)))
        .and_then(|generated| async {
          match generated {
            Either::A(p) => {
              let result = p.await?;
              Ok(result)
            }
            Either::B(_) => Ok(()),
          }
        })
        .await?;
    }
    Ok(())
  }

  #[allow(clippy::redundant_closure_for_method_calls)]
  async fn write_bundle(
    &self,
    _ctx: &rolldown_plugin::PluginContext,
    bundle: &Vec<rolldown_common::Output>,
  ) -> rolldown_plugin::HookNoopReturn {
    if let Some(cb) = &self.write_bundle {
      cb.call_async(Ok(bundle.clone().into()))
        .and_then(|written| async {
          match written {
            Either::A(p) => {
              let result = p.await?;
              Ok(result)
            }
            Either::B(_) => Ok(()),
          }
        })
        .await?;
    }
    Ok(())
  }
}
