import { afterEach, describe, expect, it } from 'bun:test'
import { mkdtemp, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import docExtract from '../doc-extract.js'
import { fixtureFile } from './helpers.ts'

const textFixture = fixtureFile('sample.txt')

describe('policy setters', () => {
  afterEach(() => {
    docExtract.setMaxFilesizeMB(42)
    docExtract.setInMemoryThresholdMB(64)
    docExtract.setMaxConcurrent(32)
    docExtract.setMaxWorkingSetMB(0)
  })

  it('setMaxFilesizeMB updates JS path validation', async () => {
    docExtract.setMaxFilesizeMB(1)
    const dir = await mkdtemp(join(tmpdir(), 'doc-extract-policy-'))
    const filePath = join(dir, 'large.txt')
    await writeFile(filePath, Buffer.alloc(2 * 1024 * 1024, 0x41))

    await expect(docExtract.extractText(filePath, 'txt')).rejects.toThrow('Input exceeds max size')
    await rm(dir, { recursive: true, force: true })
  })

  it('setMaxFilesizeMB updates native buffer validation', async () => {
    docExtract.setMaxFilesizeMB(1)
    const bytes = Buffer.alloc(2 * 1024 * 1024, 0x41)
    await expect(docExtract.extractText(bytes, 'txt')).rejects.toThrow()
  })

  it('setInMemoryThresholdMB triggers buffer spill path', async () => {
    docExtract.setInMemoryThresholdMB(1)
    const payload = Buffer.alloc(2 * 1024 * 1024, 0x41)
    Buffer.from('spill-marker\n').copy(payload, 0)

    const text = await docExtract.extractText(payload, 'txt')
    expect(text).toContain('spill-marker')
  })

  it('setMaxConcurrent accepts positive values', async () => {
    docExtract.setMaxConcurrent(4)
    const bytes = Buffer.from(await textFixture.bytes())
    const text = await docExtract.extractText(bytes)
    expect(text).toContain('CalendarTG')
  })

  it('setMaxConcurrent ignores zero', () => {
    expect(() => docExtract.setMaxConcurrent(0)).not.toThrow()
  })

  it('setMaxWorkingSetMB accepts positive values', () => {
    expect(() => docExtract.setMaxWorkingSetMB(512)).not.toThrow()
  })
})
