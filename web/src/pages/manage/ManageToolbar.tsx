import { Button } from '@mantine/core';
import { IconPlus } from '@tabler/icons-react';
import type { ReactNode } from 'react';
import classes from './ManageToolbar.module.css';

type Props = {
  addLabel: string;
  onAdd: () => void;
  children?: ReactNode;
};

export function ManageToolbar({ addLabel, onAdd, children }: Props) {
  return (
    <div className={classes.toolbar}>
      {children}
      <Button variant="filled" leftSection={<IconPlus size={16} />} onClick={onAdd}>
        {addLabel}
      </Button>
    </div>
  );
}
