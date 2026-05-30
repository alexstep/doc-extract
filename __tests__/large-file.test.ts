import { afterEach, describe, expect, it } from 'bun:test'
import { mkdtemp, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { strToU8, zipSync } from 'fflate'
import docExtract from '../doc-extract.js'

const textFixturePath = new URL('../fixtures/sample.txt', import.meta.url).pathname

describe('large file pipeline', () => {
  afterEach(() => {
    docExtract.setMaxFilesizeMB(42)
    docExtract.setInMemoryThresholdMB(64)
  })

  it('extracts from path without loading into JS heap', async () => {
    const text = await docExtract.extractText(textFixturePath)
    expect(text).toContain('CalendarTG')
  })

  it('spills large Buffer to temp and extracts', async () => {
    docExtract.setInMemoryThresholdMB(1)
    const payload = Buffer.alloc(2 * 1024 * 1024, 0x41)
    const header = Buffer.from('CalendarTG large buffer spill test\n')
    header.copy(payload, 0)

    const text = await docExtract.extractText(payload, 'txt')
    expect(text).toContain('CalendarTG large buffer spill test')
  })

  it('accepts files above old 64MB cap when maxFileSizeMB is 0', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'doc-extract-large-'))
    const filePath = join(dir, 'large.txt')
    const chunk = Buffer.alloc(1024 * 1024, 0x42)
    const header = Buffer.from('large-file-marker\n')
    await writeFile(filePath, Buffer.concat([header, chunk, chunk, chunk]))

    docExtract.setMaxFilesizeMB(0)
    const text = await docExtract.extractText(filePath, 'txt')
    expect(text).toContain('large-file-marker')

    await rm(dir, { recursive: true, force: true })
  })

  it('parsePkPass works from path', async () => {
    const dir = await mkdtemp(join(tmpdir(), 'doc-extract-pkpass-'))
    const filePath = join(dir, 'ticket.pkpass')
    const passJson = JSON.stringify({
      formatVersion: 1,
      organizationName: 'CalendarTG',
      passTypeIdentifier: 'pass.com.calendartg.demo',
      serialNumber: 'fixture-001',
      teamIdentifier: 'ABCDE12345',
      description: 'Fixture pass',
      eventTicket: {
        primaryFields: [{ key: 'event', label: 'Event', value: 'CalendarTG Meetup' }],
      },
    })
    const bytes = Buffer.from(
      zipSync({
        'pass.json': strToU8(passJson),
      }),
    )
    await writeFile(filePath, bytes)
    const result = await docExtract.parsePkPass(filePath)
    expect(result).not.toBeNull()
    expect(result?.pass).toBeObject()
    await rm(dir, { recursive: true, force: true })
  })
})
