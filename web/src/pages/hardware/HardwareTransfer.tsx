import { Alert, Button, Code, FileInput, Group, Stack, Table, Text } from '@mantine/core';
import { IconDownload, IconFileSpreadsheet, IconUpload } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useRef, useState } from 'react';
import { Trans, useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import type { components } from '../../api/schema';
import { ColumnHelp } from '../../components/ColumnHelp';
import { notifyError } from '../../components/notify';
import { ToolSection } from '../../components/ToolSection';
import classes from '../Tools.module.css';

type Preview = components['schemas']['HardwarePreview'];
type Result = components['schemas']['HardwareResult'];

const FILE_MAX = 4 * 1024 * 1024;
/** The most rows the server reads of a hardware file. */
const ROWS_MAX = 5_000;
/** Column names in translated text, as `<code>device_id</code>`. */
const CODE = { code: <Code /> };
const asCsv = { bodySerializer: (body: string) => body, headers: { 'Content-Type': 'text/csv' } };

export const HARDWARE_COLUMNS = [
  'device_id',
  'status',
  'name',
  'kind',
  'model',
  'platform',
  'plays',
  'region',
  'condition',
  'store',
  'purchased_on',
  'price',
  'currency',
  'estimated_value',
  'estimated_currency',
  'serial_number',
  'barcode',
  'notes',
  'active',
] as const;

/** Hardware to and from a CSV file: one row per device, matched by `device_id`, then by name. */
export function HardwareTransfer() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [file, setFile] = useState<File | null>(null);
  const [text, setText] = useState<string | null>(null);
  const [done, setDone] = useState<Result | null>(null);
  const chosen = useRef(0);

  const preview = useMutation({
    mutationFn: (body: string) => call(api.POST('/api/hardware/import/preview', { body, ...asCsv })),
    onError: (error) => notifyError(errorMessage(t, error)),
  });
  const run = useMutation({
    mutationFn: (body: string) => call(api.POST('/api/hardware/import', { body, ...asCsv })),
    onSuccess: (result) => {
      void queryClient.invalidateQueries();
      setDone(result);
      setFile(null);
      setText(null);
      preview.reset();
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const choose = async (next: File | null) => {
    const turn = ++chosen.current;
    setFile(next);
    setText(null);
    setDone(null);
    preview.reset();
    if (!next) return;
    if (next.size > FILE_MAX) {
      notifyError(t('errors.too_large'));
      return;
    }
    let content: string;
    try {
      content = new TextDecoder('utf-8', { fatal: true }).decode(await next.arrayBuffer());
    } catch {
      if (turn === chosen.current) notifyError(t('fileIssues.not_utf8'));
      return;
    }
    if (turn !== chosen.current) return;
    setText(content);
    preview.mutate(content);
  };

  return (
    <Stack gap="card">
      <ToolSection title={t('hardware.exportTitle')} description={t('hardware.exportText')}>
        <Group className={classes.downloads}>
          <Button component="a" href="api/hardware.csv" download leftSection={<IconDownload size={16} />}>
            {t('hardware.exportButton')}
          </Button>
          <Button
            component="a"
            href="api/hardware/template.csv"
            download
            variant="default"
            leftSection={<IconFileSpreadsheet size={16} />}
          >
            {t('import.templateButton')}
          </Button>
        </Group>
      </ToolSection>

      <ToolSection title={t('hardware.importTitle')} description={<Trans i18nKey="hardware.importText" components={CODE} />}>
        <FileInput
          label={t('import.file')}
          placeholder={t('import.filePlaceholder')}
          accept=".csv,text/csv"
          leftSection={<IconUpload size={16} />}
          value={file}
          onChange={(next) => void choose(next)}
          clearable
        />
        {done && (
          <Alert color="green" data-testid="hardware-import-done">
            {t('hardware.importDone', { created: done.created, updated: done.updated })}
          </Alert>
        )}
        {preview.isPending && <Text size="sm">{t('import.checking')}</Text>}
        {preview.data && text !== null && (
          <HardwarePreviewResult preview={preview.data} importing={run.isPending} onImport={() => run.mutate(text)} />
        )}
      </ToolSection>

      <ColumnHelp
        intro={<Trans i18nKey="hardware.columnsText" components={CODE} />}
        columns={HARDWARE_COLUMNS.map((name) => ({ name, help: t(`hardware.columnHelp.${name}`) }))}
      />
    </Stack>
  );
}

function HardwarePreviewResult({
  preview,
  importing,
  onImport,
}: {
  preview: Preview;
  importing: boolean;
  onImport: () => void;
}) {
  const { t, i18n } = useTranslation();
  const issueText = (code: string) =>
    i18n.exists(`importIssues.${code}`)
      ? t(`importIssues.${code}` as 'importIssues.required')
      : t(`errors.${code}` as 'errors.required');
  const issue = preview.file_issue;
  if (issue) {
    return (
      <Alert color="red" data-testid="hardware-file-issue">
        {t(`fileIssues.${issue.code}` as 'fileIssues.missing_column', {
          column: issue.column ?? '',
          line: issue.line ?? '',
          max: ROWS_MAX.toLocaleString(i18n.language),
        })}
      </Alert>
    );
  }
  const errors = preview.rows.filter((row) => row.issues.length > 0).length;
  const created = preview.rows.filter((row) => row.device_id == null).length;
  return (
    <Stack gap="sm" data-testid="hardware-preview">
      <Text size="sm">{t('hardware.previewSummary', { created, updated: preview.rows.length - created })}</Text>
      {errors > 0 && <Alert color="red">{t('import.hasErrors')}</Alert>}
      <Table.ScrollContainer minWidth={420}>
        <Table striped>
          <Table.Thead>
            <Table.Tr>
              <Table.Th>{t('hardware.line')}</Table.Th>
              <Table.Th>{t('common.name')}</Table.Th>
              <Table.Th>{t('hardware.change')}</Table.Th>
            </Table.Tr>
          </Table.Thead>
          <Table.Tbody>
            {preview.rows.map((row) => (
              <Table.Tr key={row.line}>
                <Table.Td>{row.line}</Table.Td>
                <Table.Td>{row.name}</Table.Td>
                <Table.Td>
                  {row.issues.length > 0 ? (
                    <Text size="sm" c="red">
                      {row.issues
                        .map((issue) => `${issue.column}: ${issueText(issue.code)}`)
                        .join(t('common.separator'))}
                    </Text>
                  ) : row.device_id == null ? (
                    t('hardware.new')
                  ) : (
                    t('hardware.update')
                  )}
                </Table.Td>
              </Table.Tr>
            ))}
          </Table.Tbody>
        </Table>
      </Table.ScrollContainer>
      <Group>
        <Button onClick={onImport} loading={importing} disabled={errors > 0 || preview.rows.length === 0}>
          {t('import.confirm')}
        </Button>
      </Group>
    </Stack>
  );
}
