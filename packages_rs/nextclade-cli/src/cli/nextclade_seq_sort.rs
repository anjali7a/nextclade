use crate::cli::nextclade_cli::{NextcladeRunOtherParams, NextcladeSortArgs};
use crate::dataset::dataset_download::download_datasets_index_json;
use crate::io::http_client::HttpClient;
use eyre::{Report, WrapErr};
use itertools::Itertools;
use log::{info, LevelFilter};
use nextclade::io::fasta::{FastaReader, FastaRecord, FastaWriter};
use nextclade::io::fs::path_to_string;
use nextclade::make_error;
use nextclade::sort::minimizer_index::{MinimizerIndexJson, MINIMIZER_INDEX_ALGO_VERSION};
use nextclade::sort::minimizer_search::{run_minimizer_search, MinimizerSearchRecord};
use nextclade::utils::option::OptionMapRefFallible;
use nextclade::utils::string::truncate;
use ordered_float::OrderedFloat;
use owo_colors::OwoColorize;
use serde::Serialize;
use std::collections::btree_map::Entry::{Occupied, Vacant};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tinytemplate::TinyTemplate;

pub fn nextclade_seq_sort(args: &NextcladeSortArgs) -> Result<(), Report> {
  check_args(args)?;

  let NextcladeSortArgs {
    server,
    proxy_config,
    input_minimizer_index_json,
    ..
  } = args;

  let verbose = log::max_level() > LevelFilter::Info;

  let minimizer_index = if let Some(input_minimizer_index_json) = &input_minimizer_index_json {
    // If a file is provided, use data from it
    MinimizerIndexJson::from_path(input_minimizer_index_json)
  } else {
    // Otherwise fetch from dataset server
    let mut http = HttpClient::new(server, proxy_config, verbose)?;
    let index = download_datasets_index_json(&mut http)?;
    let minimizer_index_path = index
      .minimizer_index
      .iter()
      .find(|minimizer_index| MINIMIZER_INDEX_ALGO_VERSION == minimizer_index.version)
      .map(|minimizer_index| &minimizer_index.path);

    if let Some(minimizer_index_path) = minimizer_index_path {
      let minimizer_index_str = http.get(minimizer_index_path)?;
      MinimizerIndexJson::from_str(String::from_utf8(minimizer_index_str)?)
    } else {
      let server_versions = index
        .minimizer_index
        .iter()
        .map(|minimizer_index| format!("'{}'", minimizer_index.version))
        .join(",");
      let server_versions = if server_versions.is_empty() {
        "none".to_owned()
      } else {
        format!(": {server_versions}")
      };

      make_error!("No compatible reference minimizer index data is found for this dataset sever. Cannot proceed. \n\nThis version of Nextclade supports index versions up to '{}', but the server has{}.\n\nTry to to upgrade Nextclade to the latest version and/or contact dataset server maintainers.", MINIMIZER_INDEX_ALGO_VERSION, server_versions)
    }
  }?;

  run(args, &minimizer_index)
}

pub fn run(args: &NextcladeSortArgs, minimizer_index: &MinimizerIndexJson) -> Result<(), Report> {
  let NextcladeSortArgs {
    input_fastas,
    output_dir,
    output,
    search_params,
    other_params: NextcladeRunOtherParams { jobs },
    ..
  } = args;

  std::thread::scope(|s| {
    const CHANNEL_SIZE: usize = 128;
    let (fasta_sender, fasta_receiver) = crossbeam_channel::bounded::<FastaRecord>(CHANNEL_SIZE);
    let (result_sender, result_receiver) = crossbeam_channel::bounded::<MinimizerSearchRecord>(CHANNEL_SIZE);

    s.spawn(|| {
      let mut reader = FastaReader::from_paths(input_fastas).unwrap();
      loop {
        let mut record = FastaRecord::default();
        reader.read(&mut record).unwrap();
        if record.is_empty() {
          break;
        }
        fasta_sender
          .send(record)
          .wrap_err("When sending a FastaRecord")
          .unwrap();
      }
      drop(fasta_sender);
    });

    for _ in 0..*jobs {
      let fasta_receiver = fasta_receiver.clone();
      let result_sender = result_sender.clone();

      s.spawn(move || {
        let result_sender = result_sender.clone();

        for fasta_record in &fasta_receiver {
          info!("Processing sequence '{}'", fasta_record.seq_name);

          let result = run_minimizer_search(&fasta_record, minimizer_index, search_params)
            .wrap_err_with(|| {
              format!(
                "When processing sequence #{} '{}'",
                fasta_record.index, fasta_record.seq_name
              )
            })
            .unwrap();

          result_sender
            .send(MinimizerSearchRecord { fasta_record, result })
            .wrap_err("When sending minimizer record into the channel")
            .unwrap();
        }

        drop(result_sender);
      });
    }

    let writer = s.spawn(move || {
      let output_dir = &output_dir;
      let output = &output;
      writer_thread(output, output_dir, result_receiver).unwrap();
    });
  });

  Ok(())
}

fn writer_thread(
  output: &Option<String>,
  output_dir: &Option<PathBuf>,
  result_receiver: crossbeam_channel::Receiver<MinimizerSearchRecord>,
) -> Result<(), Report> {
  let template = output.map_ref_fallible(move |output| -> Result<TinyTemplate, Report> {
    let mut template = TinyTemplate::new();
    template
      .add_template("output", output)
      .wrap_err_with(|| format!("When parsing template: {output}"))?;
    Ok(template)
  })?;

  println!("Suggested datasets for each sequence");

  println!("{}┐", "─".repeat(110));

  println!(
    "{:^40} │ {:^40} │ {:^10} │ {:^10} │",
    "Sequence name", "Dataset", "Score", "Num. hits"
  );

  println!("{}┤", "─".repeat(110));

  let mut writers = BTreeMap::new();
  let mut stats = BTreeMap::new();
  let mut n_undetected = 0_usize;

  for record in result_receiver {
    let datasets = record
      .result
      .datasets
      .iter()
      .sorted_by_key(|dataset| -OrderedFloat(dataset.score))
      .collect_vec();

    print!("{:40}", truncate(&record.fasta_record.seq_name, 40));

    if datasets.is_empty() {
      println!(" │ {:40} │ {:>10.3} │ {:>10} │", "undetected".red(), "", "");
      n_undetected += 1;
    }

    for (i, dataset) in datasets.into_iter().enumerate() {
      let name = &dataset.name;
      *stats.entry(name.clone()).or_insert(1) += 1;

      let names = name
        .split('/')
        .scan(PathBuf::new(), |name, component| {
          *name = name.join(component);
          Some(name.clone())
        })
        .unique()
        .map(path_to_string)
        .collect::<Result<Vec<String>, Report>>()?;

      for name in names {
        let filepath = get_filepath(&name, &template, output_dir)?;

        if let Some(filepath) = filepath {
          let writer = get_or_insert_writer(&mut writers, filepath)?;
          writer.write(&record.fasta_record.seq_name, &record.fasta_record.seq, false)?;
        }
      }

      if i != 0 {
        print!("{:40}", "");
      }

      println!(
        " │ {:40} │ {:>10.3} │ {:>10} │",
        &truncate(&dataset.name, 40),
        &dataset.score,
        &dataset.n_hits,
      );
    }

    println!("{}┤", "─".repeat(110));
  }

  println!("\n\nSuggested datasets");
  println!("{}┐", "─".repeat(67));
  println!("{:^40} │ {:^10} │ {:^10} │", "Dataset", "Num. seq", "Percent");
  println!("{}┤", "─".repeat(67));

  let total_seq = stats.values().sum::<usize>() + n_undetected;
  let stats = stats
    .into_iter()
    .sorted_by_key(|(name, n_seq)| (-(*n_seq as isize), name.clone()));
  for (name, n_seq) in stats {
    println!(
      "{:<40} │ {:>10} │ {:>9.3}% │",
      name,
      n_seq,
      100.0 * (n_seq as f64 / total_seq as f64)
    );
  }

  if n_undetected > 0 {
    println!("{}┤", "─".repeat(67));
    println!(
      "{:<40} │ {:>10} │ {:>10} │",
      "undetected".red(),
      n_undetected.red(),
      format!("{:>9.3}%", 100.0 * (n_undetected as f64 / total_seq as f64)).red()
    );
  }

  println!("{}┤", "─".repeat(67));
  println!(
    "{:>40} │ {:>10} │ {:>10} │",
    "total".bold(),
    total_seq.bold(),
    format!("{:>9.3}%", 100.0).bold()
  );
  println!("{}┘", "─".repeat(67));

  Ok(())
}

fn get_or_insert_writer(
  writers: &mut BTreeMap<PathBuf, FastaWriter>,
  filepath: impl AsRef<Path>,
) -> Result<&mut FastaWriter, Report> {
  Ok(match writers.entry(filepath.as_ref().to_owned()) {
    Occupied(e) => e.into_mut(),
    Vacant(e) => e.insert(FastaWriter::from_path(filepath)?),
  })
}

fn get_filepath(
  name: &str,
  tt: &Option<TinyTemplate>,
  output_dir: &Option<PathBuf>,
) -> Result<Option<PathBuf>, Report> {
  Ok(match (&tt, output_dir) {
    (Some(tt), None) => {
      let filepath_str = tt
        .render("output", &OutputTemplateContext { name })
        .wrap_err("When rendering output path template")?;

      Some(PathBuf::from_str(&filepath_str).wrap_err_with(|| format!("Invalid output path: '{filepath_str}'"))?)
    }
    (None, Some(output_dir)) => Some(output_dir.join(name).join("sequences.fasta")),
    _ => None,
  })
}

#[derive(Serialize)]
struct OutputTemplateContext<'a> {
  name: &'a str,
}

fn check_args(args: &NextcladeSortArgs) -> Result<(), Report> {
  let NextcladeSortArgs { output_dir, output, .. } = args;

  if output.is_some() && output_dir.is_some() {
    return make_error!(
      "The arguments `--output-dir` and `--output` cannot be used together. Remove one or the other."
    );
  }

  if let Some(output) = output {
    if !output.contains("{name}") {
      return make_error!(
        r#"
Expected `--output` argument to contain a template string containing template variable {{name}} (with curly braces), but received:

  {output}

Make sure the variable is not substituted by your shell, programming language or workflow manager. Apply proper escaping as needed.
Example for bash shell:

  --output='outputs/{{name}}/sorted.fasta.gz'

      "#
      );
    }
  }

  Ok(())
}
