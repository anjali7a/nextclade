import React from 'react'
import { ReactResizeDetectorDimensions, withResizeDetector } from 'react-resize-detector'
import { useRecoilValue } from 'recoil'
import styled from 'styled-components'

import type { AnalysisResult } from 'src/algorithms/types'
import { genomeSizeAtom } from 'src/state/results.state'
import { SequenceMarkerGap } from './SequenceMarkerGap'
import { SequenceMarkerMissing } from './SequenceMarkerMissing'
import { SequenceMarkerMutation } from './SequenceMarkerMutation'
import { SequenceMarkerUnsequencedEnd, SequenceMarkerUnsequencedStart } from './SequenceMarkerUnsequenced'
import { SequenceMarkerFrameShift } from './SequenceMarkerFrameShift'

export const SequenceViewWrapper = styled.div`
  display: flex;
  width: 100%;
  height: 30px;
  vertical-align: middle;
  margin: 0;
  padding: 0;
`

export const SequenceViewSVG = styled.svg`
  padding: 0;
  margin: 0;
  width: 100%;
  height: 100%;
`

export interface SequenceViewProps extends ReactResizeDetectorDimensions {
  sequence: AnalysisResult
}

export function SequenceViewUnsized({ sequence, width }: SequenceViewProps) {
  const { seqName, substitutions, missing, deletions, alignmentStart, alignmentEnd, frameShifts } = sequence

  const genomeSize = useRecoilValue(genomeSizeAtom)

  if (!width) {
    return (
      <SequenceViewWrapper>
        <SequenceViewSVG fill="transparent" viewBox={`0 0 10 10`} />
      </SequenceViewWrapper>
    )
  }

  const pixelsPerBase = width / genomeSize

  const mutationViews = substitutions.map((substitution) => {
    return (
      <SequenceMarkerMutation
        key={substitution.pos}
        seqName={seqName}
        substitution={substitution}
        pixelsPerBase={pixelsPerBase}
      />
    )
  })

  const missingViews = missing.map((oneMissing) => {
    return (
      <SequenceMarkerMissing
        key={oneMissing.begin}
        seqName={seqName}
        missing={oneMissing}
        pixelsPerBase={pixelsPerBase}
      />
    )
  })

  const deletionViews = deletions.map((deletion) => {
    return (
      <SequenceMarkerGap key={deletion.start} seqName={seqName} deletion={deletion} pixelsPerBase={pixelsPerBase} />
    )
  })

  const frameShiftMarkers = frameShifts.map((frameShift) => (
    <SequenceMarkerFrameShift
      key={`${frameShift.geneName}_${frameShift.nucAbs.begin}`}
      seqName={seqName}
      frameShift={frameShift}
      pixelsPerBase={pixelsPerBase}
    />
  ))

  return (
    <SequenceViewWrapper>
      <SequenceViewSVG viewBox={`0 0 ${width} 10`}>
        <rect fill="transparent" x={0} y={-10} width={genomeSize} height="30" />
        <SequenceMarkerUnsequencedStart
          seqName={seqName}
          alignmentStart={alignmentStart}
          pixelsPerBase={pixelsPerBase}
        />
        {mutationViews}
        {missingViews}
        {deletionViews}
        <SequenceMarkerUnsequencedEnd
          seqName={seqName}
          genomeSize={genomeSize}
          alignmentEnd={alignmentEnd}
          pixelsPerBase={pixelsPerBase}
        />
        {frameShiftMarkers}
      </SequenceViewSVG>
    </SequenceViewWrapper>
  )
}

export const SequenceViewUnmemoed = withResizeDetector(SequenceViewUnsized)

export const SequenceView = React.memo(SequenceViewUnmemoed)