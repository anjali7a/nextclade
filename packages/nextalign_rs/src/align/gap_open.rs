use crate::align::align::AlignPairwiseParams;
use crate::gene::gene::Gene;
use std::collections::HashMap;

pub fn get_gap_open_close_scores_flat(ref_seq: &[u8], params: &AlignPairwiseParams) -> Vec<i32> {
  let value = params.penaltyGapOpen as i32;
  let len = ref_seq.len() + 2;
  vec![value; len]
}

pub fn get_gap_open_close_scores_codon_aware(
  ref_seq: &[u8],
  gene_map: &HashMap<String, Gene>,
  params: &AlignPairwiseParams,
) -> Vec<i32> {
  let mut gap_open_close = get_gap_open_close_scores_flat(ref_seq, params);
  for (_, gene) in gene_map.iter() {
    for i in (gene.start..gene.end).step_by(3) {
      gap_open_close[i] = params.penaltyGapOpenInFrame;
      gap_open_close[i + 1] = params.penaltyGapOpenOutOfFrame;
      gap_open_close[i + 2] = params.penaltyGapOpenOutOfFrame;
    }
  }
  gap_open_close
}
