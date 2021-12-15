import { current } from 'immer'
import { WritableDraft } from 'immer/dist/types/types-external'
import { reducerWithInitialState } from 'src/state/util/fsaReducer'

import type { Gene } from 'src/algorithms/types'
import { sortResults } from 'src/helpers/sortResults'
import { runFilters } from 'src/filtering/runFilters'

import { errorDismiss } from 'src/state/error/error.actions'
import {
  algorithmRunAsync,
  resultsSortTrigger,
  setAAFilter,
  setAlgorithmGlobalStatus,
  setCladesFilter,
  setIsDirty,
  setMutationsFilter,
  setSeqNamesFilter,
  setShowGood,
  setShowErrors,
  setShowBad,
  setShowMediocre,
  setTreeResult,
  setFasta,
  setTree,
  setGeneMap,
  setQcSettings,
  setRootSeq,
  setPcrPrimers,
  removeGeneMap,
  removePcrPrimers,
  removeFasta,
  removeTree,
  removeQcSettings,
  removeRootSeq,
  addParsedSequence,
  addNextcladeResult,
  setGeneMapObject,
  setGenomeSize,
  setCurrentDataset,
  setDatasets,
  setInputUrlParams,
} from './algorithm.actions'
import {
  algorithmDefaultState,
  AlgorithmGlobalStatus,
  AlgorithmSequenceStatus,
  AlgorithmState,
} from './algorithm.state'

function removeFastaImpl(draft: WritableDraft<AlgorithmState>) {
  draft.params.raw.seqData = undefined
  draft.params.strings.queryName = undefined
  draft.params.strings.queryStr = undefined
  draft.params.errors.seqData = []
  draft.params.seqData = undefined
  return draft
}

function removeTreeImpl(draft: WritableDraft<AlgorithmState>) {
  draft.params.raw.auspiceData = undefined
  draft.params.strings.treeStr = undefined
  draft.params.errors.auspiceData = []
  return draft
}

function removeRootSeqImpl(draft: WritableDraft<AlgorithmState>) {
  draft.params.raw.rootSeq = undefined
  draft.params.strings.refStr = undefined
  draft.params.final.genomeSize = undefined
  draft.params.errors.rootSeq = []
  return draft
}

function removeQcSettingsImpl(draft: WritableDraft<AlgorithmState>) {
  draft.params.raw.qcRulesConfig = undefined
  draft.params.strings.qcConfigStr = undefined
  draft.params.errors.qcRulesConfig = []
  return draft
}

function removeGeneMapImpl(draft: WritableDraft<AlgorithmState>) {
  draft.params.raw.geneMap = undefined
  draft.params.strings.geneMapStr = undefined
  draft.params.final.geneMap = undefined
  draft.params.errors.geneMap = []
  return draft
}

function removePcrPrimersImpl(draft: WritableDraft<AlgorithmState>) {
  draft.params.raw.pcrPrimers = undefined
  draft.params.strings.pcrPrimerCsvRowsStr = undefined
  draft.params.errors.pcrPrimers = []
  return draft
}

function removeAll(draft: WritableDraft<AlgorithmState>) {
  removeFastaImpl(draft)
  removeTreeImpl(draft)
  removeRootSeqImpl(draft)
  removeQcSettingsImpl(draft)
  removeGeneMapImpl(draft)
  removePcrPrimersImpl(draft)
}

export const algorithmReducer = reducerWithInitialState(algorithmDefaultState)
  .icase(setDatasets, (draft, { defaultDatasetName, defaultDatasetNameFriendly, datasets }) => {
    draft.params.defaultDatasetName = defaultDatasetName
    draft.params.defaultDatasetNameFriendly = defaultDatasetNameFriendly
    draft.params.datasets = datasets
  })

  .icase(setCurrentDataset, (draft, dataset) => {
    removeAll(draft)
    draft.params.datasetCurrent = dataset
  })

  .icase(setInputUrlParams, (draft, urlParams) => {
    draft.params.urlParams = urlParams
  })

  .icase(setGenomeSize, (draft, { genomeSize }) => {
    draft.params.final.genomeSize = genomeSize
  })

  .icase(setGeneMapObject, (draft, { geneMap }) => {
    draft.params.final.geneMap = geneMap
  })

  .icase(addParsedSequence, (draft, { index, seqName }) => {
    draft.results[index] = {
      status: AlgorithmSequenceStatus.queued,
      id: index,
      seqName,
      result: undefined,
      query: undefined,
      queryPeptides: undefined,
      warnings: { global: [], inGenes: [] },
      errors: [],
    }
    draft.resultsFiltered = runFilters(current(draft))
  })

  .icase(addNextcladeResult, (draft, { nextcladeResult }) => {
    draft.results[nextcladeResult.index].result = nextcladeResult.analysisResult
    draft.results[nextcladeResult.index].query = nextcladeResult.query
    draft.results[nextcladeResult.index].queryPeptides = nextcladeResult.queryPeptides
    draft.results[nextcladeResult.index].warnings = nextcladeResult.warnings

    if (nextcladeResult.hasError) {
      draft.results[nextcladeResult.index].status = AlgorithmSequenceStatus.failed
      draft.results[nextcladeResult.index].errors = [nextcladeResult.error]
    } else {
      draft.results[nextcladeResult.index].status = AlgorithmSequenceStatus.done
      draft.results[nextcladeResult.index].errors = []
    }

    draft.resultsFiltered = runFilters(current(draft))
  })

  .icase(resultsSortTrigger, (draft, sorting) => {
    draft.filters.sorting = sorting
    draft.results = sortResults(current(draft).results, sorting)
    draft.resultsFiltered = runFilters(current(draft))
  })

  .icase(setSeqNamesFilter, (draft, seqNamesFilter) => {
    draft.filters.seqNamesFilter = seqNamesFilter
    draft.resultsFiltered = runFilters(current(draft))
  })

  .icase(setMutationsFilter, (draft, mutationsFilter) => {
    draft.filters.mutationsFilter = mutationsFilter
    draft.resultsFiltered = runFilters(current(draft))
  })

  .icase(setAAFilter, (draft, aaFilter) => {
    draft.filters.aaFilter = aaFilter
    draft.resultsFiltered = runFilters(current(draft))
  })

  .icase(setCladesFilter, (draft, cladesFilter) => {
    draft.filters.cladesFilter = cladesFilter
    draft.resultsFiltered = runFilters(current(draft))
  })

  .icase(setShowGood, (draft, showGood) => {
    draft.filters.showGood = showGood
    draft.resultsFiltered = runFilters(current(draft))
  })

  .icase(setShowMediocre, (draft, showMediocre) => {
    draft.filters.showMediocre = showMediocre
    draft.resultsFiltered = runFilters(current(draft))
  })

  .icase(setShowBad, (draft, showBad) => {
    draft.filters.showBad = showBad
    draft.resultsFiltered = runFilters(current(draft))
  })

  .icase(setShowErrors, (draft, showErrors) => {
    draft.filters.showErrors = showErrors
    draft.resultsFiltered = runFilters(current(draft))
  })

  // ******************

  .icase(setFasta.started, (draft, input) => {
    removeFastaImpl(draft)
    draft.params.raw.seqData = input
    draft.params.errors.seqData = []
  })

  .icase(setTree.started, (draft, input) => {
    removeTreeImpl(draft)
    draft.params.raw.auspiceData = input
    draft.params.errors.auspiceData = []
  })

  .icase(setRootSeq.started, (draft, input) => {
    removeRootSeqImpl(draft)
    draft.params.raw.rootSeq = input
    draft.params.errors.rootSeq = []
  })

  .icase(setQcSettings.started, (draft, input) => {
    removeQcSettingsImpl(draft)
    draft.params.raw.qcRulesConfig = input
    draft.params.errors.qcRulesConfig = []
  })

  .icase(setGeneMap.started, (draft, input) => {
    removeGeneMapImpl(draft)
    draft.params.raw.geneMap = input
    draft.params.errors.geneMap = []
  })

  .icase(setPcrPrimers.started, (draft, input) => {
    removePcrPrimersImpl(draft)
    draft.params.raw.pcrPrimers = input
    draft.params.errors.pcrPrimers = []
  })

  // ******************

  .icase(setFasta.done, (draft, { result: { queryStr, queryName } }) => {
    draft.params.strings.queryStr = queryStr
    draft.params.strings.queryName = queryName
    draft.params.errors.seqData = []
  })

  .icase(setTree.done, (draft, { result: { treeStr } }) => {
    draft.params.strings.treeStr = treeStr
    draft.params.errors.auspiceData = []
  })

  .icase(setRootSeq.done, (draft, { result: { refStr, refName } }) => {
    draft.params.strings.refStr = refStr
    draft.params.strings.refName = refName
    draft.params.final.genomeSize = refStr.length
    draft.params.errors.rootSeq = []
  })

  .icase(setQcSettings.done, (draft, { result: { qcConfigStr } }) => {
    draft.params.strings.qcConfigStr = qcConfigStr
    draft.params.errors.qcRulesConfig = []
  })

  .icase(setGeneMap.done, (draft, { result: { geneMapStr } }) => {
    const geneMap = JSON.parse(geneMapStr) as Gene[]
    draft.params.strings.geneMapStr = geneMapStr
    draft.params.final.geneMap = geneMap
    draft.params.errors.geneMap = []
  })

  .icase(setPcrPrimers.done, (draft, { result: { pcrPrimerCsvRowsStr } }) => {
    draft.params.strings.pcrPrimerCsvRowsStr = pcrPrimerCsvRowsStr
    draft.params.errors.pcrPrimers = []
  })

  // ******************

  .icase(setFasta.failed, (draft, { error }) => {
    removeFastaImpl(draft)
    draft.params.errors.seqData = [error]
  })

  .icase(setTree.failed, (draft, { error }) => {
    removeTreeImpl(draft)
    draft.params.errors.auspiceData = [error]
  })

  .icase(setRootSeq.failed, (draft, { error }) => {
    removeRootSeqImpl(draft)
    draft.params.errors.rootSeq = [error]
  })

  .icase(setQcSettings.failed, (draft, { error }) => {
    removeQcSettingsImpl(draft)
    draft.params.errors.qcRulesConfig = [error]
  })

  .icase(setGeneMap.failed, (draft, { error }) => {
    removeGeneMapImpl(draft)
    draft.params.errors.geneMap = [error]
  })

  .icase(setPcrPrimers.failed, (draft, { error }) => {
    removePcrPrimersImpl(draft)
    draft.params.errors.pcrPrimers = [error]
  })

  // ******************

  .icase(removeFasta, removeFastaImpl)
  .icase(removeTree, removeTreeImpl)
  .icase(removeRootSeq, removeRootSeqImpl)
  .icase(removeQcSettings, removeQcSettingsImpl)
  .icase(removeGeneMap, removeGeneMapImpl)
  .icase(removePcrPrimers, removePcrPrimersImpl)

  // ******************

  .icase(setIsDirty, (draft, isDirty) => {
    draft.status = AlgorithmGlobalStatus.idle
    draft.isDirty = isDirty
  })

  .icase(algorithmRunAsync.started, (draft) => {
    draft.status = AlgorithmGlobalStatus.idle
    draft.isDirty = false
    draft.results = []
    draft.resultsFiltered = []
    draft.treeStr = undefined
    draft.errors = []
  })

  .icase(setAlgorithmGlobalStatus, (draft, status) => {
    draft.status = status
  })

  .icase(algorithmRunAsync.trigger, (draft) => {
    draft.status = AlgorithmGlobalStatus.idle
    draft.errors = []
  })

  .icase(algorithmRunAsync.done, (draft) => {
    draft.status = AlgorithmGlobalStatus.done
    draft.errors = []
  })

  .icase(algorithmRunAsync.failed, (draft, { error }) => {
    draft.status = AlgorithmGlobalStatus.failed
    draft.errors = [error.message]
  })

  // ******************

  .icase(setTreeResult, (draft, { treeStr }) => {
    draft.treeStr = treeStr
  })

  .icase(errorDismiss, (draft) => {
    draft.errors = []
  })
