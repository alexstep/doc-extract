import { describe, expect, it } from 'bun:test'
import { strToU8, zipSync } from 'fflate'
import docExtract from '../doc-extract.js'

function buildPkPassBytes() {
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
  const strings = '"EVENT_NAME" = "CalendarTG Meetup";\n'

  return Buffer.from(
    zipSync({
      'pass.json': strToU8(passJson),
      'en.lproj/pass.strings': strToU8(strings),
      'strip.png': Uint8Array.from([0x89, 0x50, 0x4e, 0x47]),
    }),
  )
}

describe('parsePkPass', () => {
  it('returns structured pass json and strip image', async () => {
    const result = await docExtract.parsePkPass(buildPkPassBytes())
    expect(result).not.toBeNull()
    expect(result?.pass).toBeObject()
    expect((result?.pass as { eventTicket?: unknown }).eventTicket).toBeObject()
    expect(result?.localization).toContain('CalendarTG Meetup')
    expect(result?.stripImage?.startsWith('data:image/png;base64,')).toBeTrue()
  })

  it('returns null for invalid pass payload', async () => {
    const result = await docExtract.parsePkPass(Buffer.from('not-a-zip'))
    expect(result).toBeNull()
  })
})

describe('extractText pkpass', () => {
  it('returns formatted text for pkpass bytes', async () => {
    const text = await docExtract.extractText(buildPkPassBytes())
    expect(text).toContain('Apple Wallet pass')
    expect(text).toContain('"eventTicket"')
  })
})
