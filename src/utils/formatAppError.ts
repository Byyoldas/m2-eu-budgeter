import type { AppError } from '../types';

/** Human-readable message for an AppError that has no dedicated field-level display. */
export function formatAppError(error: AppError): string {
  switch (error.kind) {
    case 'Validation':
      return error.detail.map((e) => e.message).join(' ');
    case 'Calculation':
      return error.detail.message;
    case 'Persistence':
      return `Could not save your changes: ${error.detail}`;
    case 'NotFound':
      return error.detail;
    case 'NoProject':
      return 'No project is currently open.';
    case 'Internal':
      return `Something went wrong: ${error.detail}`;
    default:
      return 'An unexpected error occurred.';
  }
}
