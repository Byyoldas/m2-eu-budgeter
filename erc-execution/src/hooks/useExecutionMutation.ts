import { useCallback, useState } from 'react';
import { useExecutionStore } from '../store/executionStore';
import { appErrorMessage } from '../utils/errors';
import type { AppError, ExecutionProjectSummaryDto } from '../types';

/** Shared plumbing for every form: run an IPC mutation, push the returned
 * summary into the store, and surface any `AppError` as a display string. */
export function useExecutionMutation() {
  const updateSummary = useExecutionStore((s) => s.updateSummary);
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const run = useCallback(
    async (fn: () => Promise<ExecutionProjectSummaryDto>): Promise<boolean> => {
      setIsSubmitting(true);
      setError(null);
      try {
        const summary = await fn();
        updateSummary(summary);
        return true;
      } catch (e) {
        setError(appErrorMessage(e as AppError));
        return false;
      } finally {
        setIsSubmitting(false);
      }
    },
    [updateSummary],
  );

  return { run, error, isSubmitting };
}
