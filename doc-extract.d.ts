export interface DocExtractOptions {
  maxConcurrent?: number
  maxFileSizeMB?: number
  inMemoryThresholdMB?: number
  tempDir?: string
  debug?: boolean
}

export interface ExtractCallOptions {
  format?: string
  unknown?: 'reject' | 'text-if-likely' | 'text-lossy'
  maxFileSizeMB?: number
  inMemoryThresholdMB?: number
  tempDir?: string
  debug?: boolean
}

export interface PkPassResult {
  pass: Record<string, unknown>
  localization?: string
  stripImage?: string
}

declare class DocExtract {
  maxFileSizeMB?: number
  maxConcurrent?: number
  inMemoryThresholdMB?: number
  tempDir?: string
  debug?: boolean

  constructor(options?: DocExtractOptions)

  extractText(input: Buffer | string, formatOrOptions?: string | ExtractCallOptions): Promise<string>
  parsePkPass(input: Buffer | string, options?: Pick<ExtractCallOptions, 'maxFileSizeMB' | 'inMemoryThresholdMB' | 'tempDir'>): Promise<PkPassResult | null>

  static setMaxConcurrent(n: number): void
  static setMaxFilesizeMB(n: number): void
  static setInMemoryThresholdMB(n: number): void
  static setMaxWorkingSetMB(n: number): void
  static setTempDir(dir: string | undefined): void
  static extractText(input: Buffer | string, formatOrOptions?: string | ExtractCallOptions): Promise<string>
  static parsePkPass(input: Buffer | string, options?: Pick<ExtractCallOptions, 'maxFileSizeMB' | 'inMemoryThresholdMB' | 'tempDir'>): Promise<PkPassResult | null>
}

export default DocExtract
