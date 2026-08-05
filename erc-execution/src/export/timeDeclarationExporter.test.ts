/**
 * Tests for the EU time-declaration export. `fetch` is mocked to serve the
 * real bundled template from disk (jsdom has no dev server backing
 * `public/` during a test run), and Blob/URL/anchor are mocked the same way
 * as excelExporter.test.ts. The captured buffer is fed back into JSZip to
 * assert on the real generated document(s), rather than trusting the code
 * under test to have written what it claims.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import JSZip from 'jszip';
import { exportTimeDeclarations } from './timeDeclarationExporter';
import type { PersonDetailDto, PersonMonthDetailDto } from '../types';

const TEMPLATE_PATH = resolve(__dirname, '../../public/templates/time-declaration_en.docx');

const person: PersonDetailDto = {
  id: 'p1',
  full_name: 'Dr. Jane Doe',
  email: null,
  institution: null,
  orcid: null,
  linked_role_id: 'r1',
  linked_role_label: 'PI Role',
  actual_start_date: '2026-01-01',
  actual_end_date: null,
};

function record(overrides: Partial<PersonMonthDetailDto>): PersonMonthDetailDto {
  return {
    id: 'pm1',
    person_id: 'p1',
    project_month: 1,
    reported_months: '1',
    approved_months: '1',
    salary_cost_estimate_eur: '1000',
    calendar_year: 2026,
    calendar_month: 1,
    ...overrides,
  };
}

describe('timeDeclarationExporter', () => {
  let capturedBuffer: Uint8Array | null = null;
  let capturedFilename = '';
  // Re-wrapped via the test realm's own Uint8Array constructor: Node's
  // `fs`-produced Buffer is backed by an ArrayBuffer from a different realm
  // than vitest's jsdom environment, and JSZip's internal type detection
  // uses `instanceof ArrayBuffer` against the realm it runs in — a
  // cross-realm buffer fails that check even though the bytes are
  // identical, so it must be copied through the in-scope constructor.
  const templateBytesForComparison = new Uint8Array(readFileSync(TEMPLATE_PATH)).buffer;

  beforeEach(() => {
    capturedBuffer = null;
    capturedFilename = '';
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => ({
        ok: true,
        status: 200,
        arrayBuffer: async () => templateBytesForComparison,
      })),
    );
    vi.stubGlobal(
      'Blob',
      class {
        parts: BlobPart[];
        constructor(parts: BlobPart[]) {
          this.parts = parts;
          capturedBuffer = parts[0] as Uint8Array;
        }
      },
    );
    vi.stubGlobal('URL', { createObjectURL: vi.fn(() => 'blob:mock'), revokeObjectURL: vi.fn() });
    const realCreateElement = document.createElement.bind(document);
    vi.spyOn(document, 'createElement').mockImplementation((tag: string) =>
      tag === 'a'
        ? ({
            click: vi.fn(),
            href: '',
            set download(v: string) {
              capturedFilename = v;
            },
            get download() {
              return capturedFilename;
            },
          } as unknown as HTMLElement)
        : realCreateElement(tag),
    );
  });

  it('fills the person name and year into the real template for a single-year person', async () => {
    const result = await exportTimeDeclarations(person, [record({ project_month: 1 })]);
    expect(result.years).toEqual([2026]);
    expect(capturedFilename).toBe('time-declaration_Dr._Jane_Doe_2026.docx');

    expect(capturedBuffer).not.toBeNull();
    const zip = await JSZip.loadAsync(capturedBuffer as Uint8Array);
    const xml = await zip.file('word/document.xml')!.async('text');
    expect(xml).toContain('Dr. Jane Doe');
    expect(xml).toMatch(/<w:t[^>]*>2026<\/w:t>/);
  });

  it('injects exactly two new text runs and touches nothing else in the document', async () => {
    const originalXml = await (await JSZip.loadAsync(templateBytesForComparison))
      .file('word/document.xml')!
      .async('text');
    const originalRunCount = (originalXml.match(/<w:t[ >]/g) ?? []).length;

    await exportTimeDeclarations(person, [record({ project_month: 1 })]);
    const zip = await JSZip.loadAsync(capturedBuffer as Uint8Array);
    const xml = await zip.file('word/document.xml')!.async('text');
    const filledRunCount = (xml.match(/<w:t[ >]/g) ?? []).length;

    // Exactly one new <w:t> run per injected field (name, year) — nothing
    // else in the template (month table, footnotes reference, boilerplate)
    // was touched.
    expect(filledRunCount).toBe(originalRunCount + 2);
    // And every other file in the archive (footnotes, headers, styles) is
    // byte-identical to the template's.
    const originalZip = await JSZip.loadAsync(templateBytesForComparison);
    for (const name of Object.keys(originalZip.files)) {
      if (name === 'word/document.xml') continue;
      const before = await originalZip.file(name)!.async('uint8array');
      const after = await zip.file(name)!.async('uint8array');
      expect(after).toEqual(before);
    }
  });

  it('bundles one .docx per calendar year into a .zip when the person spans multiple years', async () => {
    const result = await exportTimeDeclarations(person, [
      record({ id: 'pm1', project_month: 1, calendar_year: 2026 }),
      record({ id: 'pm2', project_month: 13, calendar_year: 2027 }),
    ]);
    expect(result.years).toEqual([2026, 2027]);
    expect(capturedFilename).toBe('time-declarations_Dr._Jane_Doe.zip');

    const outerZip = await JSZip.loadAsync(capturedBuffer as Uint8Array);
    const names = Object.keys(outerZip.files).sort();
    expect(names).toEqual([
      'time-declaration_Dr._Jane_Doe_2026.docx',
      'time-declaration_Dr._Jane_Doe_2027.docx',
    ]);

    const docx2027 = await outerZip.file(names[1])!.async('uint8array');
    const innerZip = await JSZip.loadAsync(docx2027);
    const xml = await innerZip.file('word/document.xml')!.async('text');
    expect(xml).toMatch(/<w:t[^>]*>2027<\/w:t>/);
  });

  it('ignores records for other people (and so throws, having found none for this one)', async () => {
    await expect(
      exportTimeDeclarations(person, [
        record({ id: 'pm1', person_id: 'someone-else', calendar_year: 2030 }),
      ]),
    ).rejects.toThrow(/no person-month records yet/);
  });

  it('throws when no calendar year can be determined (no call opening date set)', async () => {
    await expect(
      exportTimeDeclarations(person, [record({ calendar_year: null, calendar_month: null })]),
    ).rejects.toThrow(/call opening date/);
  });
});
