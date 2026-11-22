import type { UseFormReturnType } from '@mantine/form';
import { useTranslation } from 'react-i18next';
import { ApiError, errorMessage } from '../api/errors';
import { notifyError } from '../components/notify';

const CONFLICT_FIELDS: Record<string, string> = { name_taken: 'name', slug_taken: 'slug' };

/**
 * Shows an API error on the form field it names (`name_taken` belongs to
 * `name`, `slug_taken` to `slug`), or as a notification when no field matches.
 */
export function useApiErrorHandler<Values extends Record<string, unknown>>(
  form: UseFormReturnType<Values>,
) {
  const { t } = useTranslation();
  return (error: unknown) => {
    const message = errorMessage(t, error);
    const field = error instanceof ApiError ? (error.field ?? CONFLICT_FIELDS[error.code]) : undefined;
    if (field && field in form.getValues()) {
      form.setFieldError(field, message);
    } else {
      notifyError(message);
    }
  };
}
