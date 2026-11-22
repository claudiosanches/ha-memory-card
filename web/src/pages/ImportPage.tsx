import { Accordion, Alert, Anchor, Badge, Button, Code, FileInput, Group, List, Select, SimpleGrid, Stack, Table, Text } from '@mantine/core';
import { IconDownload, IconFileSpreadsheet, IconUpload } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { useState } from 'react';
import { ToolSection } from '../components/ToolSection';
import { CardPage } from '../components/CardPage';
import { withColumns } from './importColumns';
import classes from './Tools.module.css';
import { useTranslation } from 'react-i18next';
import { api } from '../api/client';
import { call, errorMessage } from '../api/errors';
import type { components } from '../api/schema';
import { notifyError } from '../components/notify';
import { useStatus } from '../api/queries';

type Preview = components['schemas']['ImportPreview'];
type Summary = components['schemas']['ImportSummary'];
type Result = components['schemas']['ImportResult'];

const COLUMNS = [
  'copy_id',
  'status',
  'igdb_id',
  'game_title',
  'play_status',
  'platform',
  'region',
  'release_title',
  'product_code',
  'format',
  'edition',
  'device',
  'store',
  'condition',
  'purchased_on',
  'price',
  'currency',
  'estimated_value',
  'estimated_currency',
  'barcode',
  'notes',
  'bundle_copy_id',
] as const;

const SUMMARY_KEYS = [
  'new_games',
  'updated_games',
  'new_copies',
  'updated_copies',
  'unchanged_copies',
  'game_only_rows',
  'error_rows',
  'warning_rows',
] as const satisfies readonly (keyof Summary)[];

const asCsv = { bodySerializer: (body: string) => body, headers: { 'Content-Type': 'text/csv' } };

export function ImportPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [file, setFile] = useState<File | null>(null);
  const [text, setText] = useState<string | null>(null);
  const [done, setDone] = useState<Result | null>(null);
  const igdb = useStatus().data?.igdb_configured ?? false;

  const preview = useMutation({
    mutationFn: (body: string) => call(api.POST('/api/import/preview', { body, ...asCsv })),
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const run = useMutation({
    mutationFn: (body: string) => call(api.POST('/api/import', { body, ...asCsv })),
    onSuccess: (result) => {
      setDone(result);
      setFile(null);
      setText(null);
      preview.reset();
      void queryClient.invalidateQueries();
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const choose = async (chosen: File | null) => {
    setFile(chosen);
    setDone(null);
    preview.reset();
    if (!chosen) {
      setText(null);
      return;
    }
    const content = await chosen.text();
    setText(content);
    preview.mutate(content);
  };

  return (
    <CardPage title={t('nav.import')} description={t('import.intro')}>

      <ToolSection title={t('import.exportTitle')} description={t('import.exportText')}>
        <Group className={classes.downloads}>
          <Button component="a" href="api/export.csv" download leftSection={<IconDownload size={16} />}>
            {t('import.exportButton')}
          </Button>
          <Button
            component="a"
            href="api/import/template.csv"
            download
            variant="default"
            leftSection={<IconFileSpreadsheet size={16} />}
          >
            {t('import.templateButton')}
          </Button>
        </Group>
      </ToolSection>

      <ToolSection title={t('import.importTitle')} description={t('import.importText')}>
        <FileInput
          label={t('import.file')}
          placeholder={t('import.filePlaceholder')}
          accept=".csv,text/csv"
          leftSection={<IconUpload size={16} />}
          value={file}
          onChange={(chosen) => void choose(chosen)}
          clearable
        />

        {done && (
          <Alert color="green" data-testid="import-done">
            {t('import.done', {
              games: done.summary.new_games,
              copies: done.summary.new_copies,
              updated: done.summary.updated_copies,
            })}
          </Alert>
        )}
        {done && igdb && done.unlinked_games > 0 && (
          <Alert color="blue" data-testid="import-unlinked">
            {t('import.unlinked', { count: done.unlinked_games })}{' '}
            <Anchor renderRoot={(props) => <Link to="/igdb-matches" {...props} />} inherit>
              {t('import.reviewMatches')}
            </Anchor>
          </Alert>
        )}

        {preview.isPending && <Text size="sm">{t('import.checking')}</Text>}

        {preview.data && text !== null && preview.data.headers.length > 0 && (
          <ColumnMapping
            preview={preview.data}
            onChange={(columns) => {
              const next = withColumns(text, columns, preview.data.delimiter);
              setText(next);
              preview.mutate(next);
            }}
          />
        )}

        {preview.data && text !== null && (
          <PreviewResult
            preview={preview.data}
            importing={run.isPending}
            onImport={() => run.mutate(text)}
            onReset={() => void choose(null)}
          />
        )}
      </ToolSection>

      <ColumnHelp />
    </CardPage>
  );
}

/** How each header of the file is read, with a choice to read it as another column. */
function ColumnMapping({ preview, onChange }: { preview: Preview; onChange: (columns: (string | null)[]) => void }) {
  const { t } = useTranslation();
  const columns = preview.headers.map((h) => h.column ?? null);
  const options = [
    { value: '', label: t('import.notImported') },
    ...COLUMNS.map((column) => ({ value: column, label: column })),
  ];
  return (
    <Stack gap="xs" data-testid="column-mapping">
      <Text fw={600} size="sm">
        {t('import.columnsRead')}
      </Text>
      <Text size="xs" c="dimmed">
        {t('import.columnsReadHint')}
      </Text>
      <Table className={classes.mapping}>
        <Table.Tbody>
          {preview.headers.map((header, i) => (
            <Table.Tr key={i}>
              <Table.Td>
                <Code>{header.header || t('import.noHeader')}</Code>
              </Table.Td>
              <Table.Td>
                <Select
                  aria-label={t('import.readAs', { header: header.header })}
                  data={options}
                  value={columns[i] ?? ''}
                  allowDeselect={false}
                  onChange={(value) => onChange(columns.map((c, j) => (j === i ? value || null : c)))}
                />
              </Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </Stack>
  );
}

function PreviewResult({
  preview,
  importing,
  onImport,
  onReset,
}: {
  preview: Preview;
  importing: boolean;
  onImport: () => void;
  onReset: () => void;
}) {
  const { t } = useTranslation();
  const issue = preview.file_issue;

  if (issue) {
    return (
      <Alert color="red" data-testid="file-issue">
        {t(`fileIssues.${issue.code}` as 'fileIssues.empty_file', {
          column: issue.column ?? '',
          line: issue.line ?? '',
        })}
      </Alert>
    );
  }

  const { summary } = preview;
  const hasErrors = summary.error_rows > 0;
  const changes =
    summary.new_games +
    summary.updated_games +
    summary.new_copies +
    summary.updated_copies +
    summary.updated_releases;
  const flagged = preview.rows.filter((row) => row.issues.length > 0);

  return (
    <Stack gap="md" data-testid="import-preview">
      <SimpleGrid cols={{ base: 2, sm: 4 }}>
        {SUMMARY_KEYS.map((key) => (
          <Stack key={key} gap={0}>
            <Text size="xs" c="dimmed">
              {t(`import.summary.${key}`)}
            </Text>
            <Text fw={700} c={key === 'error_rows' && summary[key] > 0 ? 'red' : undefined} data-testid={`summary-${key}`}>
              {summary[key]}
            </Text>
          </Stack>
        ))}
      </SimpleGrid>

      {(preview.new_platforms.length > 0 || preview.new_devices.length > 0 || preview.new_stores.length > 0) && (
        <Stack gap={4}>
          <Text fw={600} size="sm">
            {t('import.willCreate')}
          </Text>
          {preview.new_platforms.length > 0 && (
            <Text size="sm">
              {t('import.newPlatforms')}: {preview.new_platforms.join(', ')}
            </Text>
          )}
          {preview.new_devices.length > 0 && (
            <Stack gap={0}>
              <Text size="sm">{t('import.newDevices')}:</Text>
              <List size="sm">
                {preview.new_devices.map((device) => (
                  <List.Item key={device.name}>
                    {t('import.newDevice', {
                      name: device.name,
                      kind: t(`deviceKind.${device.kind}`),
                      platforms: device.platforms.join(', '),
                    })}
                  </List.Item>
                ))}
              </List>
            </Stack>
          )}
          {preview.new_stores.length > 0 && (
            <Text size="sm">
              {t('import.newStores')}: {preview.new_stores.join(', ')}
            </Text>
          )}
        </Stack>
      )}

      {flagged.length > 0 && (
        <Stack gap={4}>
          <Text fw={600} size="sm">
            {t('import.issuesTitle')}
          </Text>
          <Table className={classes.issues} data-testid="import-issues">
            <Table.Thead>
              <Table.Tr>
                <Table.Th>{t('import.line')}</Table.Th>
                <Table.Th>{t('import.game')}</Table.Th>
                <Table.Th>{t('import.column')}</Table.Th>
                <Table.Th>{t('import.problem')}</Table.Th>
              </Table.Tr>
            </Table.Thead>
            <Table.Tbody>
              {flagged.flatMap((row) =>
                row.issues.map((issue) => (
                  <Table.Tr key={`${row.line}-${issue.column}-${issue.code}`}>
                    <Table.Td data-label={t('import.line')}>{row.line}</Table.Td>
                    <Table.Td data-label={t('import.game')}>{row.game_title}</Table.Td>
                    <Table.Td data-label={t('import.column')}>
                      <Code>{issue.column}</Code>
                    </Table.Td>
                    <Table.Td data-label={t('import.problem')}>
                      <Group gap="xs" wrap="nowrap">
                        <Badge size="xs" color={issue.error ? 'red' : 'yellow'}>
                          {issue.error ? t('common.error') : t('common.warning')}
                        </Badge>
                        <Text size="sm">{t(`importIssues.${issue.code}` as 'importIssues.required')}</Text>
                      </Group>
                    </Table.Td>
                  </Table.Tr>
                )),
              )}
            </Table.Tbody>
          </Table>
        </Stack>
      )}

      {hasErrors && (
        <Alert color="red">{t('import.hasErrors')}</Alert>
      )}
      {!hasErrors && changes === 0 && <Alert color="blue">{t('import.nothingToDo')}</Alert>}

      <Group>
        <Button onClick={onImport} loading={importing} disabled={hasErrors || changes === 0}>
          {t('import.confirm')}
        </Button>
        <Anchor component="button" type="button" size="sm" onClick={onReset}>
          {t('import.reset')}
        </Anchor>
      </Group>
    </Stack>
  );
}

function ColumnHelp() {
  const { t } = useTranslation();
  return (
    <Accordion variant="contained">
      <Accordion.Item value="columns">
        <Accordion.Control>{t('import.columnsTitle')}</Accordion.Control>
        <Accordion.Panel>
          <Stack gap="xs">
            <Text size="sm">{t('import.columnsText')}</Text>
            <Table>
              <Table.Tbody>
                {COLUMNS.map((column) => (
                  <Table.Tr key={column}>
                    <Table.Td>
                      <Code>{column}</Code>
                    </Table.Td>
                    <Table.Td>
                      <Text size="sm">{t(`import.columns.${column}`)}</Text>
                    </Table.Td>
                  </Table.Tr>
                ))}
              </Table.Tbody>
            </Table>
          </Stack>
        </Accordion.Panel>
      </Accordion.Item>
    </Accordion>
  );
}
