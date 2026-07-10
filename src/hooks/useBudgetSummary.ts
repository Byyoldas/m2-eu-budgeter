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
import type { BudgetSummaryDto, AppError, FieldError } from '../types';

interface MutationState {
  isLoading: boolean;
  globalError: AppError | null;
  fieldErrors: FieldError[];
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

  return { mutate, isLoading, globalError, fieldErrors, clearErrors };
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
