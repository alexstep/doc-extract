import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { describe, it } from 'node:test'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'

import docExtract from '../doc-extract.js'

const fixturesDir = join(dirname(fileURLToPath(import.meta.url)), '../fixtures')

function fixturePath(name) {
  return join(fixturesDir, name)
}

function fixtureBytes(name) {
  return readFileSync(fixturePath(name))
}

const CASES = [
  { file: 'sample.txt', needle: 'CalendarTG', format: 'txt' },
  { file: 'unicode.txt', needle: 'Привет café 日本語', format: 'txt' },
  { file: 'sample.md', needle: 'Markdown fixture', format: 'md' },
  { file: 'sample.log', needle: 'Log fixture', format: 'log' },
  { file: 'sample.csv', needle: 'Demo Event', format: 'csv' },
  { file: 'sample.tsv', needle: 'Tsv fixture', format: 'tsv' },
  { file: 'sample.html', needle: 'HTML fixture text', format: 'html' },
  { file: 'sample.xml', needle: 'CalendarTG XML Fixture', format: 'xml' },
  { file: 'sample.json', needle: 'CalendarTG Demo Event', format: 'json' },
  { file: 'sample.jsonl', needle: 'Jsonl fixture', format: 'jsonl' },
  { file: 'sample.ics', needle: 'CalendarTG Test Event', format: 'ics' },
  { file: 'sample.vcf', needle: 'CalendarTG Contact', format: 'vcf' },
  { file: 'sample.fb2', needle: 'CalendarTG FB2 fixture paragraph', format: 'fb2' },
  { file: 'sample.rtf', needle: 'Rtf fixture', format: 'rtf' },
  { file: 'sample.docx', needle: 'Docx fixture Привет', format: 'docx' },
  { file: 'sample.docm', needle: 'Docx fixture Привет', format: 'docm' },
  { file: 'sample.xlsx', needle: 'Xlsx fixture Привет', format: 'xlsx' },
  { file: 'sample.xls', needle: 'Xls fixture Привет', format: 'xls' },
  { file: 'sample.ods', needle: 'Ods fixture Привет', format: 'ods' },
  { file: 'sample.pptx', needle: 'Pptx fixture 日本語', format: 'pptx' },
  { file: 'sample.pptm', needle: 'Pptx fixture 日本語', format: 'pptm' },
  { file: 'sample.epub', needle: 'Epub fixture café', format: 'epub' },
  { file: 'sample.odt', needle: 'Odt fixture café', format: 'odt' },
  { file: 'sample1.pdf', needle: 'Sample PDF', format: 'pdf' },
]

describe('supported formats', () => {
  for (const { file, needle, format } of CASES) {
    it(`extracts ${file} from a path`, async () => {
      const text = await docExtract.extractText(fixturePath(file))
      assert.equal(typeof text, 'string')
      assert.ok(text.includes(needle), `${file} path text missing ${JSON.stringify(needle)}: ${text}`)
    })

    it(`extracts ${file} from a buffer with explicit format`, async () => {
      const text = await docExtract.extractText(fixtureBytes(file), format)
      assert.ok(text.includes(needle), `${file} buffer text missing ${JSON.stringify(needle)}: ${text}`)
    })
  }

  it('treats markdown, ical, vcard, ndjson, and htm as aliases', async () => {
    const md = await docExtract.extractText(fixtureBytes('sample.md'), 'markdown')
    assert.ok(md.includes('Markdown fixture'))

    const ical = await docExtract.extractText(fixtureBytes('sample.ics'), 'ical')
    const ifb = await docExtract.extractText(fixtureBytes('sample.ics'), 'ifb')
    assert.ok(ical.includes('CalendarTG Test Event'))
    assert.equal(ifb, ical)

    const vcard = await docExtract.extractText(fixtureBytes('sample.vcf'), 'vcard')
    assert.ok(vcard.includes('CalendarTG Contact'))

    const ndjson = await docExtract.extractText(fixtureBytes('sample.jsonl'), 'ndjson')
    assert.ok(ndjson.includes('Jsonl fixture'))

    const htm = await docExtract.extractText(fixtureBytes('sample.html'), 'htm')
    const xhtml = await docExtract.extractText(fixtureBytes('sample.html'), 'xhtml')
    assert.ok(htm.includes('HTML fixture text'))
    assert.equal(xhtml, htm)
  })
})

describe('broken and empty inputs', () => {
  const emptyOrBroken = [
    'broken/empty.pdf',
    'broken/empty.docx',
    'broken/empty.txt',
    'broken/empty.xlsx',
    'broken/truncated.pdf',
    'broken/truncated.docx',
  ]

  for (const file of emptyOrBroken) {
    it(`returns an empty string for ${file}`, async () => {
      const text = await docExtract.extractText(fixturePath(file))
      assert.equal(text, '')
    })
  }

  it('returns an empty string for an unsupported format', async () => {
    const text = await docExtract.extractText(fixtureBytes('unicode.txt'), 'exe')
    assert.equal(text, '')
  })

  it('throws when the file is missing', async () => {
    await assert.rejects(() => docExtract.extractText(fixturePath('missing-file.pdf')))
  })
})
