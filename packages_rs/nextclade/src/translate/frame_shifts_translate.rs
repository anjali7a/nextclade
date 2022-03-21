#![allow(clippy::integer_division)]

use crate::gene::gene::Gene;
use crate::io::letter::Letter;
use crate::io::nuc::Nuc;
use crate::translate::coord_map::CoordMap;
use crate::utils::range::Range;
use itertools::Itertools;

/// Find beginning nucleotide position of a deletion that immediately proceeds and adjacent to the frame shift
pub fn find_mask_begin(seq: &[Nuc], frame_shift_nuc_range_rel: &Range) -> usize {
  // From begin, rewind to find the first adjacent nuc deletion
  let mut begin = frame_shift_nuc_range_rel.begin - 1;
  if begin > 0 {
    while seq[begin as usize].is_gap() {
      begin -= 1;
    }
  }

  // `begin` now points to the nuc that is immediately before the deletion.
  // Go back one nuc to make it point to the deletion.
  begin + 1
}

/// Find ending nucleotide position of a deletion that immediately follows and adjacent to the frame shift
pub fn find_mask_end(seq: &[Nuc], frame_shift_nuc_range_rel: &Range) -> usize {
  // From end, rewind backwards to find the last adjacent nuc deletion
  let mut end = frame_shift_nuc_range_rel.end;
  while end < seq.len() && seq[end].is_gap() {
    end += 1;
  }

  // `end` now points to the nuc that is 1 past the deletion. Which is correct - we use semi-open ranges.
  end
}

pub fn find_mask(query: &[Nuc], frame_shift_nuc_range_rel: &Range) -> Range {
  Range {
    begin: find_mask_begin(query, frame_shift_nuc_range_rel),
    end: find_mask_end(query, frame_shift_nuc_range_rel),
  }
}

pub struct FrameShiftContext {
  pub codon: Range,
}

pub struct FrameShift {
  pub gene_name: String,
  pub nuc_rel: Range,
  pub nuc_abs: Range,
  pub codon: Range,
  pub gaps_leading: FrameShiftContext,
  pub gaps_trailing: FrameShiftContext,
  pub codon_mask: Range,
}

#[inline]
pub fn nuc_range_to_codon_range(range: &Range) -> Range {
  Range {
    begin: range.begin / 3,
    // Make sure the right boundary is aligned to codon boundary
    end: (range.end + (3 - range.end % 3) % 3) / 3,
  }
}

pub fn frame_shift_translate(nuc_rel_aln: &Range, query: &[Nuc], coord_map: &CoordMap, gene: &Gene) -> FrameShift {
  // Relative nuc range is in alignment coordinates. However, after insertions are stripped,
  // absolute positions may change - so in order to get absolute range, we need to convert range boundaries
  // from alignment coordinates (as in aligned reference sequence, with gaps) to reference coordinates
  // (as in the original reference coordinates, with gaps stripped).

  let gene_start_ref = gene.start;
  let gene_start_aln = coord_map.ref_to_aln_scalar(gene.start); // Gene start in alignment coordinates

  let nuc_abs_aln = nuc_rel_aln + gene_start_aln;
  let nuc_abs_ref = coord_map.aln_to_ref(&nuc_abs_aln);
  let nuc_rel_ref = &nuc_abs_ref - gene_start_ref;
  let codon = nuc_range_to_codon_range(&nuc_rel_ref);

  let mask_nuc_rel_aln = find_mask(query, nuc_rel_aln);
  let mask_nuc_abs_aln = mask_nuc_rel_aln + gene_start_aln;
  let mask_nuc_abs_ref = coord_map.aln_to_ref(&mask_nuc_abs_aln);
  let mask_nuc_rel_ref = mask_nuc_abs_ref - gene_start_ref;

  let mut codon_mask = nuc_range_to_codon_range(&mask_nuc_rel_ref);

  // Nuc mask can span beyond the gene. Prevent peptide mask overflow.
  codon_mask.end = codon_mask.end.min(gene.len() / 3);

  let gaps_leading = FrameShiftContext {
    codon: Range {
      begin: codon_mask.begin,
      end: codon.begin,
    },
  };

  let gaps_trailing = FrameShiftContext {
    codon: Range {
      begin: codon.end,
      end: codon_mask.end,
    },
  };

  FrameShift {
    gene_name: gene.gene_name.clone(),
    nuc_rel: nuc_rel_aln.clone(),
    nuc_abs: nuc_abs_ref,
    codon,
    gaps_leading,
    gaps_trailing,
    codon_mask,
  }
}

/// Converts relative nucleotide frame shifts to the final result, including
/// relative and absolute nucleotide frame shifts and relative aminoacid frame shifts
pub fn frame_shifts_translate(
  nuc_rel_frame_shifts: &[Range],
  query: &[Nuc],
  coord_map: &CoordMap,
  gene: &Gene,
) -> Vec<FrameShift> {
  nuc_rel_frame_shifts
    .iter()
    .map(|fs_nuc_rel_aln| frame_shift_translate(fs_nuc_rel_aln, query, coord_map, gene))
    .collect_vec()
}
