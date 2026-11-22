import { Alert, TextInput } from '@mantine/core';
import type { UseFormReturnType } from '@mantine/form';
import { IconAlertTriangle } from '@tabler/icons-react';
import type { TFunction } from 'i18next';
import { type ChangeEvent, type FocusEvent, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { isSlug, sanitizeSlug, slugify, trimSlug } from '../slug';

type Named = { name: string; slug: string };

/**
 * Input props for a form's name and slug. A new record's slug follows its
 * name until someone types a slug of their own.
 */
export function useSlugInputs<Values extends Named>(form: UseFormReturnType<Values>, saved?: string) {
  const named = form as unknown as UseFormReturnType<Named>;
  const [follows, setFollows] = useState(saved === undefined);
  const name = named.getInputProps('name');
  const slug = named.getInputProps('slug');
  return {
    name: {
      ...name,
      onChange: (event: ChangeEvent<HTMLInputElement>) => {
        name.onChange(event);
        if (follows) named.setFieldValue('slug', slugify(event.currentTarget.value));
      },
    },
    slug: {
      ...slug,
      onChange: (event: ChangeEvent<HTMLInputElement>) => {
        const value = sanitizeSlug(event.currentTarget.value);
        named.setFieldValue('slug', value);
        // Clearing a new record's slug hands it back to the name.
        setFollows(saved === undefined && value === '');
      },
      onBlur: (event: FocusEvent<HTMLInputElement>) => {
        named.setFieldValue('slug', trimSlug(named.getValues().slug));
        slug.onBlur?.(event);
      },
    },
  };
}

/** Form rule: a saved record needs a slug; a new one without one gets it from its name. */
export function slugRule(t: TFunction, required: boolean) {
  return (value: string) => {
    const slug = trimSlug(value);
    if (!slug) return required ? t('errors.required') : null;
    return isSlug(slug) ? null : t('errors.invalid_slug');
  };
}

type Props = ReturnType<typeof useSlugInputs>['slug'] & {
  /** The search param library links use, such as `store`. */
  param: string;
  /** The slug as saved, to warn when it changes. */
  saved?: string;
};

export function SlugInput({ param, saved, ...props }: Props) {
  const { t } = useTranslation();
  const value = String(props.value ?? '');
  return (
    <>
      <TextInput
        label={t('slug.label')}
        description={t('slug.description', { link: `?${param}=${trimSlug(value) || '…'}` })}
        autoCapitalize="none"
        autoCorrect="off"
        spellCheck={false}
        {...props}
      />
      {saved !== undefined && trimSlug(value) !== saved && (
        <Alert color="yellow" p="xs" icon={<IconAlertTriangle size={16} />} data-testid="slug-changed">
          {t('slug.changed')}
        </Alert>
      )}
    </>
  );
}
