import React, { useCallback } from 'react'

import { connect } from 'react-redux'
import { Button } from 'reactstrap'

import { setCurrentDataset } from 'src/state/algorithm/algorithm.actions'
import { useTranslationSafe } from 'src/helpers/useTranslationSafe'
import { selectCurrentDataset } from 'src/state/algorithm/algorithm.selectors'

import type { DatasetFlat } from 'src/algorithms/types'
import type { State } from 'src/state/reducer'
import styled from 'styled-components'
import { DatasetInfo } from './DatasetInfo'

export const CurrentDatasetInfoContainer = styled.div`
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  margin-bottom: 1rem;
`

export const CurrentDatasetInfoHeader = styled.section`
  display: flex;
  margin-bottom: 0.5rem;
`

export const CurrentDatasetInfoBody = styled.section`
  display: flex;
  flex-direction: column;
  margin: 0;
  padding: 0.5rem;
  border: 1px #ccc9 solid;
  border-radius: 5px;
`

export const ChangeButton = styled(Button)`
  margin: auto 0;
  height: 2.1rem;
  min-width: 100px;
`

export const CustomizeButton = styled(Button)`
  height: 1.6rem;
  font-size: 0.85rem;
  padding: 0;
`

export interface DatasetCurrentProps {
  dataset?: DatasetFlat
  setCurrentDataset(dataset: DatasetFlat | undefined): void
}

const mapStateToProps = (state: State) => ({
  dataset: selectCurrentDataset(state),
})

const mapDispatchToProps = {
  setCurrentDataset,
}

export const DatasetCurrent = connect(mapStateToProps, mapDispatchToProps)(DatasetCurrentDisconnected)

export function DatasetCurrentDisconnected({ dataset, setCurrentDataset }: DatasetCurrentProps) {
  const { t } = useTranslationSafe()

  const onChangeClicked = useCallback(() => {
    setCurrentDataset(undefined)
  }, [setCurrentDataset])

  const onCustomizeClicked = useCallback(() => {}, [])

  if (!dataset) {
    return null
  }

  return (
    <CurrentDatasetInfoContainer>
      <CurrentDatasetInfoHeader>
        <h3>{t('Selected pathogen')}</h3>
        <ChangeButton className="ml-auto" type="button" color="secondary" onClick={onChangeClicked}>
          {t('Change')}
        </ChangeButton>
      </CurrentDatasetInfoHeader>

      <CurrentDatasetInfoBody>
        <DatasetInfo dataset={dataset} />
        <div>
          <CustomizeButton type="button" color="link" onClick={onCustomizeClicked}>
            {t('Customize (advanced)')}
          </CustomizeButton>
        </div>
      </CurrentDatasetInfoBody>
    </CurrentDatasetInfoContainer>
  )
}
