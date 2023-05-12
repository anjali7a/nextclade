use clap::{AppSettings, Parser, ValueHint};
use ctor::ctor;
use eyre::Report;
use log::LevelFilter;
use nextclade::gene::cds::{Cds, CdsSegment};
use nextclade::gene::gene::{Gene, GeneStrand};
use nextclade::gene::protein::{Protein, ProteinSegment};
use nextclade::io::file::create_file_or_stdout;
use nextclade::io::json::json_write_impl;
use nextclade::utils::global_init::global_init;
use nextclade::utils::global_init::setup_logger;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::io::{stdout, Write};
use std::path::PathBuf;

#[ctor]
fn init() {
  global_init();
}

#[derive(Parser, Debug)]
#[clap(name = "generate_jsonschema", trailing_var_arg = true)]
#[clap(author, version)]
#[clap(global_setting(AppSettings::DeriveDisplayOrder))]
#[clap(verbatim_doc_comment)]
pub struct FeaturemapArgs {
  /// Path to output directory
  #[clap(long, short = 'o')]
  #[clap(value_hint = ValueHint::DirPath)]
  pub output: Option<PathBuf>,
}

fn write_jsonschema<T: JsonSchema>(output: &Option<PathBuf>) -> Result<(), Report> {
  let writer: Box<dyn Write + Send> = match output {
    None => Box::new(stdout()),
    Some(output) => {
      let filename = format!("{}.json", T::schema_name());
      create_file_or_stdout(&output.join(filename))?
    }
  };

  let schema = schema_for!(T);
  json_write_impl(writer, &schema)
}

fn main() -> Result<(), Report> {
  let args = FeaturemapArgs::parse();
  setup_logger(LevelFilter::Warn);

  write_jsonschema::<_SchemaRoot>(&args.output)?;

  Ok(())
}

// The doc comment will appear in the schema file.
/// AUTOGENERATED! DO NOT EDIT! This schema file is generated automatically from Rust types.
/// The topmost schema definition is a dummy container. Disregard it.
/// See the actual types in the `definitions` property.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct _SchemaRoot {
  _1: Gene,
  _2: GeneStrand,
  _3: Protein,
  _4: ProteinSegment,
  _5: Cds,
  _6: CdsSegment,
}
