/**
 * EU Grants "Declaration of Days Worked on a Project" (time declaration)
 * export, for the Personnel screen. Uses the real EU template (bundled as a
 * static asset at `public/templates/time-declaration_en.docx`) rather than
 * recreating its layout, since this is an audit-relevant document and
 * fidelity to the official form matters.
 *
 * Only the person's name and the calendar year are filled in. Days
 * worked / Work Packages / all signature fields are deliberately left
 * blank for manual completion: the template's own footnote defines "1 day"
 * as the beneficiary's own standard working-day length, which this app has
 * no way to know, so there's no reliable way to convert the tracked
 * FTE-fraction person-month data into an actual day count — and the app
 * doesn't track which Work Packages a person worked on in a given month at
 * all. Inventing either would put a fabricated number on an audit document.
 */

import JSZip from 'jszip';
import type { PersonDetailDto, PersonMonthDetailDto } from '../types';

const TEMPLATE_URL = '/templates/time-declaration_en.docx';

// Unique w14:paraId values (from the bundled template's word/document.xml)
// identifying the two blank table cells this export fills in. If the
// template asset is ever replaced with a different version, these must be
// re-verified against the new file's XML.
const NAME_CELL_PARA_ID = '7B374A11';
const YEAR_CELL_PARA_ID = '7B374A05';

function escapeXml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

/** Inserts a text run at the end of the paragraph identified by `paraId`. */
function injectIntoParagraph(xml: string, paraId: string, value: string): string {
  const markerIdx = xml.indexOf(`w14:paraId="${paraId}"`);
  if (markerIdx === -1) {
    throw new Error(`Time declaration template: could not find the expected field (${paraId}).`);
  }
  const closeIdx = xml.indexOf('</w:p>', markerIdx);
  if (closeIdx === -1) {
    throw new Error(`Time declaration template: malformed document structure near ${paraId}.`);
  }
  const run = `<w:r><w:t xml:space="preserve">${escapeXml(value)}</w:t></w:r>`;
  return xml.slice(0, closeIdx) + run + xml.slice(closeIdx);
}

async function loadTemplateZip(): Promise<JSZip> {
  const res = await fetch(TEMPLATE_URL);
  if (!res.ok) {
    throw new Error(`Failed to load the time declaration template (HTTP ${res.status}).`);
  }
  const buf = await res.arrayBuffer();
  return JSZip.loadAsync(buf);
}

/** Returns the filled .docx as raw bytes (not a Blob) so callers can feed it
 * straight into another JSZip archive (for the multi-year bundle case)
 * without round-tripping through Blob's async read APIs. */
async function buildFilledDocxBytes(personName: string, year: number): Promise<Uint8Array> {
  const zip = await loadTemplateZip();
  const docXmlFile = zip.file('word/document.xml');
  if (!docXmlFile) {
    throw new Error('Time declaration template is missing word/document.xml.');
  }
  let xml = await docXmlFile.async('text');
  xml = injectIntoParagraph(xml, NAME_CELL_PARA_ID, personName);
  xml = injectIntoParagraph(xml, YEAR_CELL_PARA_ID, String(year));
  zip.file('word/document.xml', xml);
  return zip.generateAsync({ type: 'uint8array' });
}

function downloadBlob(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

function sanitizeFilenamePart(s: string): string {
  return s.replace(/[^a-zA-Z0-9._-]+/g, '_');
}

/**
 * Exports one filled time declaration per calendar year the person has a
 * person-month record in (bundled as a .zip if there's more than one).
 * Throws if no calendar year can be determined at all, i.e. the project has
 * no call opening date set (see `progress_engine::project_month_to_calendar`
 * on the Rust side) — there is nothing to generate in that case.
 */
export async function exportTimeDeclarations(
  person: PersonDetailDto,
  personMonths: PersonMonthDetailDto[],
): Promise<{ years: number[] }> {
  const recordsForPerson = personMonths.filter((r) => r.person_id === person.id);
  if (recordsForPerson.length === 0) {
    throw new Error(
      `${person.full_name} has no person-month records yet — add at least one before exporting a time declaration.`,
    );
  }

  const years = Array.from(
    new Set(recordsForPerson.map((r) => r.calendar_year).filter((y): y is number => y != null)),
  ).sort((a, b) => a - b);

  if (years.length === 0) {
    throw new Error(
      'No calendar year could be determined for this person’s records — this project has no call opening date set, so project months can’t be mapped to real years.',
    );
  }

  const DOCX_MIME = 'application/vnd.openxmlformats-officedocument.wordprocessingml.document';

  if (years.length === 1) {
    const bytes = await buildFilledDocxBytes(person.full_name, years[0]);
    downloadBlob(
      new Blob([bytes as BlobPart], { type: DOCX_MIME }),
      `time-declaration_${sanitizeFilenamePart(person.full_name)}_${years[0]}.docx`,
    );
    return { years };
  }

  const bundle = new JSZip();
  for (const year of years) {
    const bytes = await buildFilledDocxBytes(person.full_name, year);
    bundle.file(`time-declaration_${sanitizeFilenamePart(person.full_name)}_${year}.docx`, bytes);
  }
  const zipBytes = await bundle.generateAsync({ type: 'uint8array' });
  downloadBlob(
    new Blob([zipBytes as BlobPart], { type: 'application/zip' }),
    `time-declarations_${sanitizeFilenamePart(person.full_name)}.zip`,
  );
  return { years };
}
