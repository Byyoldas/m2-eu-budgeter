/**
 * CFS (Certificate on Financial Statements) prompt modal.
 *
 * Shown when the budget exceeds €430,000 and no CFS item exists.
 * User can:
 * 1. Add a CFS cost item (enters amount + year)
 * 2. Dismiss the warning (sets cfs_warning_dismissed)
 */

import { useState } from 'react';
import { useBudgetSummary } from '../hooks/useBudgetSummary';
import { addCfsItem, dismissCfsWarning } from '../ipc/commands';
import { useProjectStore } from '../store/projectStore';

interface CFSModalProps {
  open: boolean;
  onClose: () => void;
}

export function CFSModal({ open, onClose }: CFSModalProps) {
  const projectConfig = useProjectStore((s) => s.projectConfig);
  const { mutate, isLoading } = useBudgetSummary();

  const [amount, setAmount] = useState('');
  const [year, setYear] = useState(1);
  const [amountError, setAmountError] = useState('');

  const duration = projectConfig?.duration_years ?? 5;

  const handleAdd = async () => {
    const n = parseFloat(amount);
    if (isNaN(n) || n <= 0) {
      setAmountError('Enter a valid amount greater than zero.');
      return;
    }
    setAmountError('');
    const result = await mutate(() => addCfsItem(amount, year));
    if (result) onClose();
  };

  const handleDismiss = async () => {
    const result = await mutate(() => dismissCfsWarning());
    if (result) onClose();
  };

  if (!open) return null;

  return (
    <div className="modal-overlay" role="dialog" aria-modal="true">
      <div className="modal-box">
        <div className="modal-header">
          <span className="modal-icon">⚠</span>
          <h2 className="modal-title">Certificate on Financial Statements Required</h2>
        </div>

        <div className="modal-body">
          <p>
            Your requested EU contribution exceeds <strong>€430,000</strong>. Under ERC rules,
            you must include a <em>Certificate on Financial Statements (CFS)</em> as an auditing cost
            in category C3 (Other Direct Costs).
          </p>
          <p>
            Enter the CFS cost below. The auditor's fee is typically <strong>€6,000–€15,000</strong>.
            Please confirm with your institution.
          </p>

          <div className="form-row">
            <div className="form-field">
              <label htmlFor="cfs-amount" className="form-label required">CFS Cost (€)</label>
              <input
                id="cfs-amount"
                type="number"
                value={amount}
                min={0}
                step="any"
                placeholder="e.g. 10000"
                className={`form-input${amountError ? ' form-input--error' : ''}`}
                onChange={(e) => setAmount(e.target.value)}
              />
              {amountError && <span className="form-error">{amountError}</span>}
            </div>

            <div className="form-field">
              <label htmlFor="cfs-year" className="form-label required">Project Year</label>
              <select
                id="cfs-year"
                value={year}
                onChange={(e) => setYear(Number(e.target.value))}
                className="form-input"
              >
                {Array.from({ length: duration }, (_, i) => i + 1).map((y) => (
                  <option key={y} value={y}>Year {y}</option>
                ))}
              </select>
            </div>
          </div>
        </div>

        <div className="modal-footer">
          <button className="btn btn--ghost" onClick={handleDismiss} disabled={isLoading}>
            Dismiss (I'll handle this manually)
          </button>
          <button className="btn btn--primary" onClick={handleAdd} disabled={isLoading}>
            {isLoading ? 'Adding…' : 'Add CFS Cost Item'}
          </button>
        </div>
      </div>
    </div>
  );
}
