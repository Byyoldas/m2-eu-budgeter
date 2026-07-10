/**
 * Hook for safe mutation + summary update pattern.
 *
 * Usage:
 *   const { mutate, isLoading, error } = useBudgetSummary();
 *   const result = await mutate(() => addPersonnelRole(input));
 *
 * On success: updates the global summary in the store, clears error.
 * On error: sets field-level or global error, does NOT update summary.
 */

import { useState, useCallback } from 'react';
import { useProjectStore } from '../store/projectStore';
import { formatAppError } from '../utils/formatAppError';
import type { BudgetSummaryDto, AppError, FieldError } from '../types';

interface MutationState {
  isLoading: boolean;
  globalError: AppError | null;
  fieldErrors: FieldError[];
  /**
   * Human-readable message for errors that have no dedicated field to display next to
   * (a global/entity-level error, e.g. "only one PI allowed" or a backend Persistence
   * failure). Null when there is nothing to show beyond per-field messages.
   */
  formError: string | null;
  clearErrors: () => void;
}

export function useBudgetSummary(): MutationState & {
  mutate: (fn: () => Promise<BudgetSummaryDto>) => Promise<BudgetSummaryDto | null>;
} {
  const setSummary = useProjectStore((s) => s.setSummary);
  const [isLoading, setIsLoading] = useState(false);
  const [globalError, setGlobalError] = useState<AppError | null>(null);
  const [fieldErrors, setFieldErrors] = useState<FieldError[]>([]);

  const clearErrors = useCallback(() => {
    setGlobalError(null);
    setFieldErrors([]);
  }, []);

  const mutate = useCallback(
    async (fn: () => Promise<BudgetSummaryDto>): Promise<BudgetSummaryDto | null> => {
      setIsLoading(true);
      clearErrors();
      try {
        const summary = await fn();
        setSummary(summary);
        return summary;
      } catch (err) {
        const appError = err as AppError;
        if (appError.kind === 'Validation') {
          setFieldErrors(appError.detail);
        } else {
          setGlobalError(appError);
        }
        return null;
      } finally {
        setIsLoading(false);
      }
    },
    [setSummary, clearErrors]
  );

  const entityErrors = fieldErrors.filter((e) => !e.field);
  const formError = globalError
    ? formatAppError(globalError)
    : entityErrors.length > 0
      ? entityErrors.map((e) => e.message).join(' ')
      : null;

  return { mutate, isLoading, globalError, fieldErrors, formError, clearErrors };
}

/**
 * Hook for preview operations (no mutation, no summary update).
 */
export function usePreview<T>(): {
  preview: (fn: () => Promise<T>) => Promise<T | null>;
  isLoading: boolean;
  error: AppError | null;
} {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<AppError | null>(null);

  const preview = useCallback(async (fn: () => Promise<T>): Promise<T | null> => {
    setIsLoading(true);
    setError(null);
    try {
      return await fn();
    } catch (err) {
      setError(err as AppError);
      return null;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { preview, isLoading, error };
}
