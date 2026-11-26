import { Accordion, Code, Stack, Table, Text } from '@mantine/core';
import { useTranslation } from 'react-i18next';

/** A folded table of a CSV file's columns and what each one accepts. */
export function ColumnHelp({ intro, columns }: { intro: string; columns: { name: string; help: string }[] }) {
  const { t } = useTranslation();
  return (
    <Accordion variant="contained">
      <Accordion.Item value="columns">
        <Accordion.Control>{t('import.columnsTitle')}</Accordion.Control>
        <Accordion.Panel>
          <Stack gap="xs">
            <Text size="sm">{intro}</Text>
            <Table>
              <Table.Tbody>
                {columns.map(({ name, help }) => (
                  <Table.Tr key={name}>
                    <Table.Td>
                      <Code>{name}</Code>
                    </Table.Td>
                    <Table.Td>
                      <Text size="sm">{help}</Text>
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
