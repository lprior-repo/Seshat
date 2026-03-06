import { readFileSync, writeFileSync, existsSync, readdirSync } from "node:fs";
import { join } from "node:path";
import type { RedQueenTrace, SeedCorpusEntry } from "./types";

const CORPUS_DIR = join(process.cwd(), "diagram_tool", "e2e", "redqueen", "corpus");

function ensureCorpusDir(): void {
  if (!existsSync(CORPUS_DIR)) {
    const { mkdirSync } = require("node:fs");
    mkdirSync(CORPUS_DIR, { recursive: true });
  }
}

export function loadCorpusEntry(id: string): SeedCorpusEntry | null {
  const filePath = join(CORPUS_DIR, `${id}.json`);
  if (!existsSync(filePath)) {
    return null;
  }
  try {
    const content = readFileSync(filePath, "utf8");
    return JSON.parse(content) as SeedCorpusEntry;
  } catch {
    return null;
  }
}

export function saveCorpusEntry(entry: SeedCorpusEntry): void {
  ensureCorpusDir();
  const filePath = join(CORPUS_DIR, `${entry.id}.json`);
  writeFileSync(filePath, JSON.stringify(entry, null, 2), "utf8");
}

export function listCorpusEntries(): ReadonlyArray<SeedCorpusEntry> {
  if (!existsSync(CORPUS_DIR)) {
    return [];
  }
  const files = readdirSync(CORPUS_DIR).filter((f) => f.endsWith(".json"));
  return files.map((file) => {
    const content = readFileSync(join(CORPUS_DIR, file), "utf8");
    return JSON.parse(content) as SeedCorpusEntry;
  });
}

export function promoteTraceToCorpus(
  trace: RedQueenTrace,
  failureReason: string,
): SeedCorpusEntry {
  const id = `seed-${trace.seed}-wave${trace.wave}-${Date.now()}`;
  const entry: SeedCorpusEntry = {
    id,
    trace,
    promotedAt: new Date().toISOString(),
    failureReason,
    fixedAt: null,
  };
  saveCorpusEntry(entry);
  return entry;
}

export function markEntryFixed(id: string): void {
  const entry = loadCorpusEntry(id);
  if (entry) {
    const fixedEntry: SeedCorpusEntry = {
      ...entry,
      fixedAt: new Date().toISOString(),
    };
    saveCorpusEntry(fixedEntry);
  }
}

export function getUnfixedEntries(): ReadonlyArray<SeedCorpusEntry> {
  return listCorpusEntries().filter((e) => e.fixedAt === null);
}

export function getFixedEntries(): ReadonlyArray<SeedCorpusEntry> {
  return listCorpusEntries().filter((e) => e.fixedAt !== null);
}

export function tracesForReplay(): ReadonlyArray<RedQueenTrace> {
  return getFixedEntries().map((e) => e.trace);
}

export function tracesForCurrentWave(wave: 1 | 2 | 3): ReadonlyArray<RedQueenTrace> {
  return listCorpusEntries()
    .filter((e) => e.trace.wave === wave && e.fixedAt === null)
    .map((e) => e.trace);
}
