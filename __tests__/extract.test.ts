import { describe, expect, it } from 'bun:test'
import { mkdtemp, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import docExtract from '../doc-extract.js'
import { fixtureFile, fixturePath } from './helpers.ts'

const textFixture = fixtureFile('sample.txt')
const icsFixture = fixtureFile('sample.ics')
const csvFixture = fixtureFile('sample.csv')
const htmlFixture = fixtureFile('sample.html')
const xmlFixture = fixtureFile('sample.xml')
const jsonFixture = fixtureFile('sample.json')
const vcfFixture = fixtureFile('sample.vcf')
const fb2Fixture = fixtureFile('sample.fb2')

describe('DocExtract.extractText', () => {
  it('extracts txt without format', async () => {
    const value = await docExtract.extractText(Buffer.from(await textFixture.bytes()))
    expect(value).toContain('CalendarTG')
  })

  it('extracts txt from path', async () => {
    const value = await docExtract.extractText(fixturePath('sample.txt'))
    expect(value).toContain('CalendarTG')
  })

  it('extracts ics with explicit format', async () => {
    const value = await docExtract.extractText(Buffer.from(await icsFixture.bytes()), 'ics')
    expect(value).toContain('Summary: CalendarTG Test Event')
  })

  it('extracts csv', async () => {
    const value = await docExtract.extractText(Buffer.from(await csvFixture.bytes()), 'csv')
    expect(value).toContain('Demo Event')
  })

  it('extracts html', async () => {
    const value = await docExtract.extractText(Buffer.from(await htmlFixture.bytes()), 'html')
    expect(value).toContain('HTML fixture text')
  })

  it('extracts xml', async () => {
    const value = await docExtract.extractText(Buffer.from(await xmlFixture.bytes()), 'xml')
    expect(value).toContain('CalendarTG XML Fixture')
  })

  it('extracts json', async () => {
    const value = await docExtract.extractText(Buffer.from(await jsonFixture.bytes()), 'json')
    expect(value).toContain('CalendarTG Demo Event')
  })

  it('extracts vcf', async () => {
    const value = await docExtract.extractText(Buffer.from(await vcfFixture.bytes()), 'vcard')
    expect(value).toContain('FN: CalendarTG Contact')
    expect(value).toContain('BDAY: 1990-05-01')
  })

  it('extracts fb2', async () => {
    const value = await docExtract.extractText(Buffer.from(await fb2Fixture.bytes()), 'fb2')
    expect(value).toContain('CalendarTG FB2 fixture paragraph')
  })

  it('supports ical alias', async () => {
    const bytes = Buffer.from(await icsFixture.bytes())
    const ics = await docExtract.extractText(bytes, 'ics')
    const ical = await docExtract.extractText(bytes, 'ical')
    expect(ical).toBe(ics)
  })

  it('supports markdown alias', async () => {
    const bytes = Buffer.from(await textFixture.bytes())
    const txt = await docExtract.extractText(bytes, 'txt')
    const md = await docExtract.extractText(bytes, 'md')
    expect(md).toBe(txt)
  })

  it('returns empty string for unsupported format', async () => {
    const bytes = Buffer.from(await textFixture.bytes())
    const value = await docExtract.extractText(bytes, 'exe')
    expect(value).toBe('')
  })

  it('rejects payload larger than limit via native buffer check', async () => {
    docExtract.setMaxFilesizeMB(1)
    const bytes = Buffer.alloc(2 * 1024 * 1024, 1)
    await expect(docExtract.extractText(bytes, 'txt')).rejects.toThrow()
    docExtract.setMaxFilesizeMB(42)
  })

  it('rejects path larger than limit via JS validation', async () => {
    docExtract.setMaxFilesizeMB(1)
    const dir = await mkdtemp(join(tmpdir(), 'doc-extract-limit-'))
    const filePath = join(dir, 'big.txt')
    await writeFile(filePath, Buffer.alloc(2 * 1024 * 1024, 0x42))
    await expect(docExtract.extractText(filePath, 'txt')).rejects.toThrow('Input exceeds max size')
    await rm(dir, { recursive: true, force: true })
    docExtract.setMaxFilesizeMB(42)
  })

  it('instance maxFileSizeMB applies per call', async () => {
    const custom = new docExtract({ maxFileSizeMB: 1 })
    const bytes = Buffer.alloc(2 * 1024 * 1024, 1)
    await expect(custom.extractText(bytes, 'txt')).rejects.toThrow()
  })
})

describe('DocExtract instance', () => {
  it('runs extractText on instance', async () => {
    const custom = new docExtract({ maxConcurrent: 2 })
    const bytes = Buffer.from(await textFixture.bytes())
    const value = await custom.extractText(bytes)
    expect(value).toContain('CalendarTG')
  })
})
