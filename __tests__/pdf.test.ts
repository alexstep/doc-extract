import { describe, expect, it } from 'bun:test'
import docExtract from '../doc-extract.js'
import { fixtureFile, fixturePath } from './helpers.ts'

const sample1Path = fixturePath('sample1.pdf')
const sample2Path = fixturePath('sample2.pdf')
const sample1Bytes = Buffer.from(await fixtureFile('sample1.pdf').bytes())
const sample2Bytes = Buffer.from(await fixtureFile('sample2.pdf').bytes())

describe('PDF fixtures', () => {
  it('extracts sample1.pdf from path with auto-detect', async () => {
    const text = await docExtract.extractText(sample1Path)
    expect(text).toContain('Sample PDF')
    expect(text).toContain('Lorem ipsum')
    expect(text.length).toBeGreaterThan(500)
  })

  it('extracts sample1.pdf from buffer', async () => {
    const text = await docExtract.extractText(sample1Bytes)
    expect(text).toContain('This is a simple PDF')
    expect(text).toContain('Fun fun fun')
  })

  it('extracts sample1.pdf with explicit pdf format', async () => {
    const text = await docExtract.extractText(sample1Bytes, 'pdf')
    expect(text).toContain('Pellentesque  sit  amet  lectus')
  })

  it('returns empty string for sample2.pdf (scanned image PDF without text layer)', async () => {
    const text = await docExtract.extractText(sample2Path)
    expect(text).toBe('')
  })

  it('returns empty string for sample2.pdf from buffer', async () => {
    const text = await docExtract.extractText(sample2Bytes)
    expect(text).toBe('')
  })
})
