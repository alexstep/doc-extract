'use strict'

process.env.NAPI_RS_FORCE_WASI = 'error'

const assert = require('node:assert/strict')
const { readFileSync } = require('node:fs')
const { join } = require('node:path')

const docExtract = require('../doc-extract.js')

const fixturePath = join(__dirname, '../fixtures/sample.txt')
const fixtureBytes = readFileSync(fixturePath)

async function main() {
  const fromBuffer = await docExtract.extractText(fixtureBytes)
  assert.match(fromBuffer, /CalendarTG/, 'WASI: extract from buffer')

  const fromPath = await docExtract.extractText(fixturePath)
  assert.match(fromPath, /CalendarTG/, 'WASI: extract from path')

  console.log('WASI smoke tests passed')
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})
