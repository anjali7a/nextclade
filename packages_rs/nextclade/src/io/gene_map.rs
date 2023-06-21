use crate::features::feature_tree::FeatureTree;
use crate::features::feature_type::style_for_feature_type;
use crate::features::gene_map::convert_feature_tree_to_gene_map;
use crate::gene::cds::{Cds, CdsSegment, WrappingPart};
use crate::gene::gene::Gene;
use crate::gene::protein::{Protein, ProteinSegment};
use crate::io::file::open_file_or_stdin;
use crate::io::yaml::yaml_parse;
use crate::utils::error::report_to_string;
use crate::utils::string::truncate_with_ellipsis;
use crate::{make_error, make_internal_report};
use eyre::{eyre, Report, WrapErr};
use itertools::{max, Itertools};
use log::warn;
use num::Integer;
use num_traits::clamp;
use owo_colors::OwoColorize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::Display;
use std::io::Write;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[must_use]
pub struct GeneMap {
  pub genes: BTreeMap<String, Gene>,
}

impl GeneMap {
  pub fn new() -> Self {
    Self::from_genes(BTreeMap::<String, Gene>::new())
  }

  pub fn from_genes(genes: BTreeMap<String, Gene>) -> Self {
    Self { genes }
  }

  pub fn from_feature_tree(feature_tree: &FeatureTree) -> Result<Self, Report> {
    convert_feature_tree_to_gene_map(feature_tree)
  }

  pub fn from_file<P: AsRef<Path>>(filename: P) -> Result<Self, Report> {
    let filename = filename.as_ref();
    let mut file = open_file_or_stdin(&Some(filename))?;
    let mut buf = vec![];
    {
      file.read_to_end(&mut buf)?;
    }
    Self::from_str(String::from_utf8(buf)?).wrap_err_with(|| eyre!("When reading file: {filename:?}"))
  }

  // TODO: rename this function, because it handles more than GFF3
  pub fn from_str(content: impl AsRef<str>) -> Result<Self, Report> {
    let content = content.as_ref();
    let gene_map_yaml: Result<GeneMap, Report> = Self::from_yaml_str(content);
    let gene_map_gff: Result<GeneMap, Report> = Self::from_gff3_str(content);

    let gene_map = match (gene_map_yaml, gene_map_gff) {
      (Err(json_err), Err(gff_err)) => {
        return make_error!("Attempted to parse the genome annotation as JSON and as GFF, but both attempts failed:\nJSON error: {}\n\nGFF3 error: {}\n",
          report_to_string(&json_err),
          report_to_string(&gff_err),
        )
      },
      (Ok(gene_map), _) => gene_map,
      (_, Ok(gene_map)) => gene_map,
    };

    gene_map.validate()?;
    Ok(gene_map)
  }

  fn from_yaml_str(content: impl AsRef<str>) -> Result<Self, Report> {
    yaml_parse(content.as_ref())
  }

  fn from_gff3_str(content: impl AsRef<str>) -> Result<Self, Report> {
    Self::from_feature_tree(&FeatureTree::from_gff3_str(content.as_ref())?)
  }

  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.genes.is_empty()
  }

  #[must_use]
  pub fn len(&self) -> usize {
    self.genes.len()
  }

  #[must_use]
  pub fn contains(&self, gene_name: &str) -> bool {
    self.genes.contains_key(gene_name)
  }

  pub fn get(&self, gene_name: &str) -> Result<&Gene, Report> {
    self
      .genes
      .get(gene_name)
      .ok_or_else(|| make_internal_report!("Gene is expected to be present, but not found: '{gene_name}'"))
  }

  pub fn get_cds<S: AsRef<str>>(&self, cds_name: S) -> Result<&Cds, Report> {
    let cds_name = cds_name.as_ref();
    self
      .genes
      .iter()
      .find_map(|(_, gene)| gene.cdses.iter().find(|cds| cds.name == cds_name))
      .ok_or_else(|| {
        make_internal_report!("When looking up a CDS translation: CDS '{cds_name}' is expected, but not found")
      })
  }

  pub fn iter_genes(&self) -> impl Iterator<Item = (&String, &Gene)> + '_ {
    self.genes.iter()
  }

  pub fn iter_genes_mut(&mut self) -> impl Iterator<Item = (&String, &mut Gene)> + '_ {
    self.genes.iter_mut()
  }

  pub fn into_iter_genes(self) -> impl Iterator<Item = (String, Gene)> {
    self.genes.into_iter()
  }

  pub fn genes(&self) -> impl Iterator<Item = &Gene> + '_ {
    self.genes.values()
  }

  pub fn iter_cdses(&self) -> impl Iterator<Item = &Cds> + '_ {
    self.genes.iter().flat_map(|(_, gene)| gene.cdses.iter())
  }

  pub fn iter_cdses_mut(&mut self) -> impl Iterator<Item = &mut Cds> + '_ {
    self.genes.iter_mut().flat_map(|(_, gene)| gene.cdses.iter_mut())
  }

  pub fn into_iter_cdses(self) -> impl Iterator<Item = Cds> {
    self.genes.into_iter().flat_map(|(_, gene)| gene.cdses.into_iter())
  }

  pub fn cdses(&self) -> impl Iterator<Item = &Cds> + '_ {
    self.genes.iter().flat_map(|(_, gene)| gene.cdses.iter())
  }

  pub fn validate(&self) -> Result<(), Report> {
    self.iter_cdses().try_for_each(|cds| {
      cds.len().is_multiple_of(&3).then_some(()).ok_or_else(|| {
        let segment_lengths = cds.segments.iter().map(CdsSegment::len).join("+");
        let n_segments = cds.segments.len();
        eyre!(
          "Length of a CDS is expected to be divisible by 3, but the length of CDS '{}' is {} \
          (it consists of {n_segments} fragment(s) of length(s) {segment_lengths}). \
          This is likely a mistake in genome annotation.",
          cds.name,
          cds.len()
        )
      })
    })?;

    Ok(())
  }
}

/// Filters gene map according to the list of requested genes.
///
/// Here are the possible combinations:
///
/// | --genemap  | --genes |                 behavior                   |
/// |------------|---------|--------------------------------------------|
/// |     +      |    +    | Take only specified genes                  |
/// |     +      |         | Take all genes                             |
/// |            |    +    | Error                                      |
/// |            |         | Skip translation and codon penalties       |
pub fn filter_gene_map(gene_map: Option<GeneMap>, genes: &Option<Vec<String>>) -> Result<GeneMap, Report> {
  match (gene_map, genes) {
    // Both gene map and list of genes are provided. Retain only requested genes.
    (Some(gene_map), Some(genes)) => {
      let gene_map: BTreeMap<String, Gene> = gene_map
        .into_iter_genes()
        .filter(|(gene_name, ..)| genes.contains(gene_name))
        .collect();

      let requested_genes_not_in_genemap = get_requested_genes_not_in_genemap(&gene_map, genes);
      if !requested_genes_not_in_genemap.is_empty() {
        warn!(
          "The following genes were requested through `--genes` \
           but not found in the gene map: \
           `{requested_genes_not_in_genemap}`",
        );
      }
      Ok(GeneMap::from_genes(gene_map))
    }

    // Only gene map is provided. Take all the genes.
    (Some(gene_map), None) => Ok(gene_map),

    // Gene list is provided, but no gene map. This is illegal.
    (None, Some(_)) => {
      make_error!(
        "List of genes via '--genes' can only be specified \
         when a gene map (genome annotation) is provided"
      )
    }

    // Nothing is provided. Create an empty gene map.
    // This disables codon-aware alignment, translation, AA mutations, frame shifts, and everything else that relies
    // on gene information.
    (None, None) => Ok(GeneMap::new()),
  }
}

fn get_requested_genes_not_in_genemap(gene_map: &BTreeMap<String, Gene>, genes: &[String]) -> String {
  genes
    .iter()
    .filter(|&gene_name| !gene_map.contains_key(gene_name))
    .join("`, `")
}

const INDENT: &str = " ";
const INDENT_WIDTH: usize = 2;

pub fn gene_map_to_string(gene_map: &GeneMap) -> Result<String, Report> {
  let mut buf = Vec::<u8>::new();
  {
    format_gene_map(&mut buf, gene_map)?;
  }
  Ok(String::from_utf8(buf)?)
}

pub fn format_gene_map<W: Write>(w: &mut W, gene_map: &GeneMap) -> Result<(), Report> {
  let max_gene_name_len = gene_map
    .iter_genes()
    .map(|(_, gene)| gene.name_and_type().len() + INDENT_WIDTH)
    .max()
    .unwrap_or_default();

  let max_cds_name_len = gene_map
    .iter_genes()
    .flat_map(|(_, gene)| &gene.cdses)
    .map(|cds| cds.name_and_type().len() + INDENT_WIDTH * 2)
    .max()
    .unwrap_or_default();

  let max_cds_segment_name_len = gene_map
    .iter_genes()
    .flat_map(|(_, gene)| &gene.cdses)
    .flat_map(|cds| &cds.segments)
    .map(|seg| seg.name_and_type().len() + INDENT_WIDTH * 3)
    .max()
    .unwrap_or_default();

  let max_protein_name_len = gene_map
    .iter_genes()
    .flat_map(|(_, gene)| &gene.cdses)
    .flat_map(|cds| &cds.proteins)
    .map(|protein| protein.name_and_type().len() + INDENT_WIDTH * 3)
    .max()
    .unwrap_or_default();

  let max_protein_segment_name_len = gene_map
    .iter_genes()
    .flat_map(|(_, gene)| &gene.cdses)
    .flat_map(|cds| &cds.proteins)
    .flat_map(|protein| &protein.segments)
    .map(|seg| seg.name_and_type().len() + INDENT_WIDTH * 4)
    .max()
    .unwrap_or_default();

  let max_name_len = clamp(
    max([
      max_gene_name_len,
      max_cds_name_len,
      max_cds_segment_name_len,
      max_protein_name_len,
      max_protein_segment_name_len,
    ])
    .unwrap_or_default(),
    0,
    100,
  );

  writeln!(
    w,
    "{:max_name_len$} │ s │  c  │  start  │   end   │   nucs  │    codons   │",
    "Genome",
  )?;

  for (_, gene) in gene_map
    .iter_genes()
    .sorted_by_key(|(_, gene)| (gene.range.begin, gene.range.end, &gene.name))
  {
    write_gene(w, max_name_len, gene)?;
    for cds in &gene.cdses {
      write_cds(w, max_name_len, cds)?;
      for cds_segment in &cds.segments {
        write_cds_segment(w, max_name_len, cds_segment)?;
      }
      for protein in &cds.proteins {
        write_protein(w, max_name_len, protein)?;
        for protein_segment in &protein.segments {
          write_protein_segment(w, max_name_len, protein_segment)?;
        }
      }
    }
  }
  Ok(())
}

fn write_gene<W: Write>(w: &mut W, max_name_len: usize, gene: &Gene) -> Result<(), Report> {
  let Gene { exceptions, .. } = gene;

  let indent_width = INDENT_WIDTH;
  let indent = INDENT.repeat(indent_width);
  let max_name_len = max_name_len.saturating_sub(indent_width);
  let name = truncate_with_ellipsis(gene.name_and_type(), max_name_len);
  let exceptions = exceptions.join(", ");
  writeln!(
    w,
    "{indent}{:max_name_len$} │   │     │         │         │         │             │ {exceptions}",
    name.style(style_for_feature_type("gene")?)
  )?;

  Ok(())
}

fn write_cds<W: Write>(w: &mut W, max_name_len: usize, cds: &Cds) -> Result<(), Report> {
  let indent_width = INDENT_WIDTH * 2;
  let indent = INDENT.repeat(indent_width);
  let max_name_len = max_name_len.saturating_sub(indent_width);
  let name = truncate_with_ellipsis(cds.name_and_type(), max_name_len);
  let nuc_len = cds.len();
  let codon_len = format_codon_length(nuc_len);
  let exceptions = cds.exceptions.join(", ");
  writeln!(
    w,
    "{indent}{:max_name_len$} │   │     │         │         │ {nuc_len:>7} │ {codon_len:>11} │ {exceptions}",
    name.style(style_for_feature_type("cds")?)
  )?;

  Ok(())
}

fn write_cds_segment<W: Write>(w: &mut W, max_name_len: usize, cds_segment: &CdsSegment) -> Result<(), Report> {
  let CdsSegment {
    range,
    strand,
    exceptions,
    ..
  } = cds_segment;

  let indent_width = INDENT_WIDTH * 3;
  let indent = INDENT.repeat(indent_width);
  let max_name_len = max_name_len.saturating_sub(indent_width);
  let name = truncate_with_ellipsis(cds_segment.name_and_type(), max_name_len);
  let start = range.begin.green();
  let end = range.end.red();
  let nuc_len = cds_segment.len();
  let codon_len = format_codon_length(nuc_len);
  let exceptions = exceptions.join(", ");
  let wrap = match cds_segment.wrapping_part {
    WrappingPart::NonWrapping => "   ".to_owned(),
    WrappingPart::WrappingStart => "\u{21BB} 0".to_owned(),
    WrappingPart::WrappingCentral(i) => format!("\u{21BB} {i}"),
    WrappingPart::WrappingEnd(i) => format!("\u{21BB} {i}"),
  };
  writeln!(
    w,
    "{indent}{:max_name_len$} │ {strand:} │ {wrap:} │ {start:>7} │ {end:>7} │ {nuc_len:>7} │ {codon_len:>11} │ {exceptions}",
    name.style(style_for_feature_type("cds segment")?)
  )?;

  Ok(())
}

fn write_protein<W: Write>(w: &mut W, max_name_len: usize, protein: &Protein) -> Result<(), Report> {
  let indent_width = INDENT_WIDTH * 3;
  let indent = INDENT.repeat(indent_width);
  let max_name_len = max_name_len.saturating_sub(indent_width);
  let name = truncate_with_ellipsis(protein.name_and_type(), max_name_len);
  writeln!(
    w,
    "{indent}{:max_name_len$} │   │     │         │         │         │             │",
    name.style(style_for_feature_type("protein")?)
  )?;

  Ok(())
}

fn write_protein_segment<W: Write>(
  w: &mut W,
  max_name_len: usize,
  protein_segment: &ProteinSegment,
) -> Result<(), Report> {
  let ProteinSegment {
    range,
    exceptions,
    ..
  } = protein_segment;

  let indent_width = INDENT_WIDTH * 4;
  let indent = INDENT.repeat(indent_width);
  let max_name_len = max_name_len.saturating_sub(indent_width);
  let name = truncate_with_ellipsis(protein_segment.name_and_type(), max_name_len);
  let start = range.begin;
  let end = range.end;
  let nuc_len = range.len();
  let codon_len = format_codon_length(nuc_len);
  let exceptions = exceptions.join(", ");
  writeln!(
    w,
    "{indent}{:max_name_len$} │   │     │ {start:>7} │ {end:>7} │ {nuc_len:>7} │ {codon_len:>11} │ {exceptions}",
    name.style(style_for_feature_type("protein segment")?)
  )?;

  Ok(())
}

pub fn format_codon_length(nuc_len: usize) -> String {
  let codons = nuc_len / 3;
  let codons_decimal = match nuc_len % 3 {
    0 => "     ",
    1 => " +1/3",
    2 => " +2/3",
    _ => unreachable!(),
  };
  format!("{codons}{codons_decimal}")
}
