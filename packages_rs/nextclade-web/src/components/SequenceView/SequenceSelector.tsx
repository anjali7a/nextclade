// noinspection JSUnusedLocalSymbols
import type { Cds, Gene, Protein } from '_SchemaRoot'
import { isEmpty } from 'lodash'
import React, { useCallback, useMemo } from 'react'
import ReactSelect, { StylesConfig } from 'react-select'
import { FilterOptionOption } from 'react-select/dist/declarations/src/filters'
import type { FormatOptionLabelMeta } from 'react-select/dist/declarations/src/Select'
import { Theme } from 'react-select/dist/declarations/src/types'
import { Badge } from 'reactstrap'
import styled from 'styled-components'
import { viewedGeneAtom } from 'src/state/seqViewSettings.state'
import { useRecoilState, useRecoilValue } from 'recoil'
import { GENE_OPTION_NUC_SEQUENCE } from 'src/constants'
import { useTranslationSafe } from 'src/helpers/useTranslationSafe'
import { genesAtom } from 'src/state/results.state'

const menuPortalTarget = typeof document === 'object' ? document.body : null

export interface Option {
  value: string
  color?: string
  gene?: Gene
  cds?: Cds
  protein?: Protein
  isDisabled?: boolean
}

export function SequenceSelector() {
  const genes = useRecoilValue(genesAtom)
  const [viewedGene, setViewedGene] = useRecoilState(viewedGeneAtom)

  const { options, defaultOption } = useMemo(() => {
    return prepareOptions(genes)
  }, [genes])

  const option = useMemo((): Option => {
    if (viewedGene === GENE_OPTION_NUC_SEQUENCE) {
      return { value: GENE_OPTION_NUC_SEQUENCE }
    }
    return options.find((option) => option.cds?.name === viewedGene) ?? defaultOption
  }, [defaultOption, options, viewedGene])

  const onChange = useCallback(
    (option: Option | null) => {
      if (option?.value === GENE_OPTION_NUC_SEQUENCE) {
        setViewedGene(GENE_OPTION_NUC_SEQUENCE)
      }

      if (option?.cds?.name) {
        setViewedGene(option.cds?.name)
      }
    },
    [setViewedGene],
  )

  const filterOptions = useCallback((candidate: FilterOptionOption<Option>, searchTerm: string): boolean => {
    if (candidate.value === GENE_OPTION_NUC_SEQUENCE) {
      return true
    }
    if (!isEmpty(searchTerm)) {
      return (
        (candidate.data.gene?.name?.split(' ').some((word) => word.includes(searchTerm)) ||
          candidate.data.cds?.name?.split(' ').some((word) => word.includes(searchTerm)) ||
          candidate.data.protein?.name?.split(' ').some((word) => word.includes(searchTerm))) ??
        false
      )
    }
    return true
  }, [])

  const reactSelectTheme = useCallback((theme: Theme): Theme => {
    return {
      ...theme,
      borderRadius: 2,
      spacing: {
        ...theme.spacing,
        menuGutter: 0,
      },
      colors: {
        ...theme.colors,
      },
    }
  }, [])

  const reactSelectStyles = useMemo((): StylesConfig<Option, false> => {
    return {
      menuPortal: (base) => ({ ...base, zIndex: 9999 }),
      menuList: (base) => ({ ...base, fontSize: '1rem' }),
      option: (base) => ({ ...base, fontSize: '1rem' }),
      singleValue: (base) => ({ ...base, fontSize: '1rem' }),
    }
  }, [])

  return (
    <div className="d-flex w-100">
      <InnerWrapper>
        <ReactSelect
          name="sequence-view-gene-dropdown"
          options={options}
          filterOption={filterOptions}
          formatOptionLabel={OptionLabel}
          isMulti={false}
          value={option}
          onChange={onChange}
          menuPortalTarget={menuPortalTarget}
          styles={reactSelectStyles}
          theme={reactSelectTheme}
          maxMenuHeight={500}
        />
      </InnerWrapper>
    </div>
  )
}

const InnerWrapper = styled.div`
  width: 100%;
  min-width: 100px;
  max-width: 300px;
  margin: 0.5rem auto;
`

function OptionLabel(option: Option, meta: FormatOptionLabelMeta<Option>) {
  if (option.gene && option.cds) {
    return <OptionLabelGeneAndCds gene={option.gene} cds={option.cds} />
  }

  if (option.gene) {
    return <OptionLabelGene gene={option.gene} />
  }

  if (option.cds) {
    return <OptionLabelCds cds={option.cds} isMenu={meta.context === 'menu'} />
  }

  if (option.protein) {
    return <OptionLabelProtein protein={option.protein} isMenu={meta.context === 'menu'} />
  }

  if (option.value === GENE_OPTION_NUC_SEQUENCE) {
    return <OptionLabelFullGenome isMenu={meta.context === 'menu'} />
  }

  return null
}

function OptionLabelFullGenome({ isMenu: _ }: { isMenu?: boolean }) {
  const { t } = useTranslationSafe()
  return (
    <Indent>
      <Badge color="success" className="mr-1 px-2 py-1">
        {t('Full genome')}
      </Badge>
    </Indent>
  )
}

function OptionLabelGene({ gene }: { gene: Gene; isMenu?: boolean }) {
  const { t } = useTranslationSafe()
  return (
    <Indent>
      <Badge color="secondary" className="mr-1 px-2 py-1">
        {t('Gene')}
      </Badge>
      <span className="text-body">{gene.name}</span>
    </Indent>
  )
}

function OptionLabelGeneAndCds({ gene }: { gene: Gene; cds: Cds; isMenu?: boolean }) {
  const { t } = useTranslationSafe()
  return (
    <Indent>
      <Badge className="mr-1 px-2 py-1">{t('Gene')}</Badge>
      <Badge color="primary" className="mr-1">
        {t('CDS')}
      </Badge>
      <span>{gene.name}</span>
    </Indent>
  )
}

function OptionLabelCds({ cds, isMenu = false }: { cds: Cds; isMenu?: boolean }) {
  const { t } = useTranslationSafe()
  return (
    <Indent indent={isMenu && 1}>
      <Badge color="primary" className="mr-1 px-2 py-1">
        {t('CDS')}
      </Badge>
      <span>{cds.name}</span>
    </Indent>
  )
}

function OptionLabelProtein({ protein, isMenu = false }: { protein: Protein; isMenu?: boolean }) {
  const { t } = useTranslationSafe()
  return (
    <Indent indent={isMenu && 2}>
      <Badge className="mr-1">{t('Protein')}</Badge>
      <span>{protein.name}</span>
    </Indent>
  )
}

// noinspection CssReplaceWithShorthandSafely
const Indent = styled.div<{ indent?: number | boolean }>`
  margin: 0;
  margin-left: ${(props) => (Number(props.indent) ?? 0) * 1.5}rem;
`

function prepareOptions(genes: Gene[]) {
  const options: Option[] = [{ value: GENE_OPTION_NUC_SEQUENCE }]

  if (isEmpty(genes)) {
    return { options, defaultOption: options[0] }
  }

  const defaultCds = genes[0].cdses[0]
  let defaultOption: Option = {
    value: defaultCds.name,
    cds: defaultCds,
  }

  // eslint-disable-next-line no-loops/no-loops
  for (const gene of genes) {
    if (gene.cdses.length === 1) {
      options.push({
        value: `gene-${gene.name}`,
        gene,
        cds: gene.cdses[0],
      })
    } else {
      options.push({
        value: `gene-${gene.name}`,
        gene,
        isDisabled: true,
      })

      // eslint-disable-next-line no-loops/no-loops
      for (const cds of gene.cdses) {
        const option: Option = {
          value: `cds-${cds.name}`,
          cds,
        }
        defaultOption = option
        options.push(option)

        // eslint-disable-next-line no-loops/no-loops
        for (const protein of cds.proteins) {
          options.push({
            value: `protein-${protein.name}`,
            protein,
            isDisabled: true,
          })
        }
      }
    }
  }

  return { options, defaultOption }
}
