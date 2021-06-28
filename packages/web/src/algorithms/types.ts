/* eslint-disable @typescript-eslint/no-empty-interface */
import type { Tagged } from 'src/helpers/types'

/** Type-safe representation of a nucleotide */
export type Nucleotide = Tagged<string, 'Nucleotide'>

/** Type-safe representation of an aminoacid */
export type Aminoacid = Tagged<string, 'Aminoacid'>

/** Represents a numeric interval bounded by begin and end. Similar to `Span`, but different representation. */
export interface Range {
  begin: number
  end: number
}

/** Represents a numeric interval bounded by start and length. Similar to `Range`, but different representation. */
export interface Span {
  start: number
  length: number
}

export interface NucleotideLocation {
  pos: number
  nuc: Nucleotide
}

export interface NucleotideSubstitution {
  pos: number
  refNuc: Nucleotide
  queryNuc: Nucleotide
  pcrPrimersChanged: PcrPrimer[]
  aaSubstitutions: AminoacidSubstitution[]
  aaDeletions: AminoacidDeletion[]
}

export interface NucleotideDeletion extends Span {
  aaSubstitutions: AminoacidSubstitution[]
  aaDeletions: AminoacidDeletion[]
}

export interface NucleotideInsertion {
  pos: number
  ins: string
}

export interface NucleotideMissing extends Range {}

export interface CharacterRange<Letter> extends Range {
  character: Letter
}

export type NucleotideRange = CharacterRange<Nucleotide>
export type AminoacidRange = CharacterRange<Aminoacid>

export interface GeneAminoacidRange {
  geneName: string
  character: Aminoacid
  ranges: AminoacidRange[]
  length: number
}

export type Clades = Record<string, NucleotideLocation[]>

export interface CladesGrouped {
  pos: number
  subs: Record<string, string[]>
}

export interface AminoacidSubstitution {
  refAA: Aminoacid
  codon: number
  queryAA: Aminoacid
  gene: string
  codonNucRange: Range
  refContext: string
  queryContext: string
  contextNucRange: Range
  nucSubstitutions: NucleotideSubstitution[]
  nucDeletions: NucleotideDeletion[]
}

export interface AminoacidDeletion {
  gene: string
  refAA: Aminoacid
  codon: number
  codonNucRange: Range
  refContext: string
  queryContext: string
  contextNucRange: Range
  nucSubstitutions: NucleotideSubstitution[]
  nucDeletions: NucleotideDeletion[]
}

export interface PcrPrimer {
  name: string
  target: string
  source: string
  rootOligonuc: string
  primerOligonuc: string
  range: Range
  nonACGTs: NucleotideLocation[]
}

export interface PcrPrimerChange {
  primer: PcrPrimer
  substitutions: NucleotideSubstitution[]
}

export interface QCRulesConfigMissingData {
  enabled: boolean
  missingDataThreshold: number
  scoreBias: number
}

export interface QCRulesConfigMixedSites {
  enabled: boolean
  mixedSitesThreshold: number
}

export interface QCRulesConfigPrivateMutations {
  enabled: boolean
  typical: number
  cutoff: number
}

export interface QCRulesConfigSnpClusters {
  enabled: boolean
  windowSize: number
  clusterCutOff: number
  scoreWeight: number
}

export interface QCRulesConfigFrameShifts {
  enabled: boolean
}

export interface QCRulesConfigStopCodons {
  enabled: boolean
}

export interface QcConfig {
  schemaVersion: string
  missingData: QCRulesConfigMissingData
  mixedSites: QCRulesConfigMixedSites
  privateMutations: QCRulesConfigPrivateMutations
  snpClusters: QCRulesConfigSnpClusters
  frameShifts: QCRulesConfigFrameShifts
  stopCodons: QCRulesConfigStopCodons
}

export interface Virus {
  name: string
  minimalLength: number
  queryStr: string
  treeJson: string
  refFastaStr: string
  qcConfigRaw: string
  qcConfigJson: QcConfig
  geneMapStrRaw: string
  pcrPrimersStrRaw: string
}

export interface ClusteredSNPs {
  start: number
  end: number
  numberOfSNPs: number
}

export enum QcStatus {
  good = 'good',
  mediocre = 'mediocre',
  bad = 'bad',
}

export interface QcResultMixedSites {
  score: number
  status: QcStatus
  totalMixedSites: number
  mixedSitesThreshold: number
}

export interface ClusteredSnp {
  start: number
  end: number
  numberOfSNPs: number
}

export interface QcResultSnpClusters {
  score: number
  status: QcStatus
  totalSNPs: number
  clusteredSNPs: ClusteredSnp[]
}

export interface QcResultMissingData {
  score: number
  status: QcStatus
  totalMissing: number
  missingDataThreshold: number
}

export interface QcResultPrivateMutations {
  score: number
  status: QcStatus
  total: number
  excess: number
  cutoff: number
}

export interface FrameShift {
  geneName: string
}

export interface QcResultFrameShifts {
  score: number
  status: QcStatus
  frameShifts: FrameShift[]
  totalFrameShifts: number
}

export interface StopCodonLocation {
  geneName: string
  codon: number
}

export interface QcResultStopCodons {
  score: number
  status: QcStatus
  stopCodons: StopCodonLocation[]
  totalStopCodons: number
}

export interface QcResult {
  missingData?: QcResultMissingData
  mixedSites?: QcResultMixedSites
  privateMutations?: QcResultPrivateMutations
  snpClusters?: QcResultSnpClusters
  frameShifts?: QcResultFrameShifts
  stopCodons?: QcResultStopCodons
  overallScore: number
  overallStatus: QcStatus
}

export interface AnalysisResult {
  seqName: string
  substitutions: NucleotideSubstitution[]
  totalSubstitutions: number
  insertions: NucleotideInsertion[]
  totalInsertions: number
  deletions: NucleotideDeletion[]
  totalDeletions: number
  missing: NucleotideMissing[]
  totalMissing: number
  nonACGTNs: NucleotideRange[]
  totalNonACGTNs: number
  aaSubstitutions: AminoacidSubstitution[]
  totalAminoacidSubstitutions: number
  aaDeletions: AminoacidDeletion[]
  totalAminoacidDeletions: number
  unknownAaRanges: GeneAminoacidRange[]
  totalUnknownAa: number
  alignmentStart: number
  alignmentEnd: number
  alignmentScore: number
  alignedQuery: string
  nucleotideComposition: Record<string, number>
  pcrPrimerChanges: PcrPrimerChange[]
  totalPcrPrimerChanges: number
  clade: string
  qc: QcResult
}

export interface Peptide {
  name: string
  seq: string
}

/** Represents a named interval in the genome */
export interface Gene {
  geneName: string
  color: string
  start: number
  end: number
  length: number
  frame: number
  strand: string
}

export interface SequenceParserResult {
  index: number
  seqName: string
  seq: string
}

export interface GeneWarning {
  geneName: string
  message: string
}

export interface Warnings {
  global: string[]
  inGenes: GeneWarning[]
}

export interface DatasetsSettings {
  defaultDataset: string
}

export interface DatasetVersion {
  datetime: string
  comment: string
  compatibility: {
    'nextclade-cli-version': {
      min?: string
      max?: string
    }
    'nextclade-web-version': {
      min?: string
      max?: string
    }
  }
  files: string[]
}

export interface Dataset {
  'name': string
  'name-friendly': string
  'description': string
  'versions': DatasetVersion[]
}

export interface DatasetsJson {
  settings: DatasetsSettings
  datasets: Dataset[]
}
