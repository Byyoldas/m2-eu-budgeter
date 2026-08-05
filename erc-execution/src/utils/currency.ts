/** Formats a `Decimal`-as-string EUR amount as `€ 12,345.67` for display. */
export function fmtEur(v: string): string {
  const n = parseFloat(v);
  return isNaN(n) ? '€ 0.00' : `€ ${n.toLocaleString('en-GB', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}
