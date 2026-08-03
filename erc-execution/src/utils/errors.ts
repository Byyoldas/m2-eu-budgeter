import type { AppError } from '../types';

/** Human-readable message for any `AppError` variant, for inline display. */
export function appErrorMessage(error: AppError): string {
  switch (error.kind) {
    case 'Validation':
      return error.detail.map((f) => f.message).join(' ');
    case 'Calculation':
      return error.detail.message;
    case 'Persistence':
    case 'NotFound':
    case 'Internal':
      return error.detail;
    case 'NoProject':
      return 'No project is open.';
    default:
      return 'An unexpected error occurred.';
  }
}
