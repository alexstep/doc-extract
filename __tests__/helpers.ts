import { fileURLToPath } from 'node:url'

const fixturesDir = new URL('../fixtures/', import.meta.url)

export function fixturePath(name: string): string {
  return fileURLToPath(new URL(name, fixturesDir))
}

export function fixtureFile(name: string) {
  return Bun.file(new URL(name, fixturesDir))
}
