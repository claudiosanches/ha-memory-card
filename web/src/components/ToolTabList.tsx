import { Select, Tabs, useTabsContext } from '@mantine/core';

type Props = {
  label: string;
  tabs: { value: string; label: string }[];
};

export function ToolTabList({ label, tabs }: Props) {
  const { value, onChange } = useTabsContext();

  return (
    <>
      <Select
        hiddenFrom="xs"
        mb="lg"
        size="md"
        aria-label={label}
        styles={{
          input: { textOverflow: 'ellipsis' },
          option: { whiteSpace: 'normal', overflowWrap: 'anywhere' },
        }}
        data={tabs}
        value={value}
        onChange={onChange}
        allowDeselect={false}
      />
      <Tabs.List visibleFrom="xs" aria-label={label} mb="lg">
        {tabs.map((tab) => (
          <Tabs.Tab key={tab.value} value={tab.value}>{tab.label}</Tabs.Tab>
        ))}
      </Tabs.List>
    </>
  );
}
