import { Stack, Title } from '@mantine/core';
import { Description } from './Description';

type Props = {
  title: string;
  description?: string;
  titleClassName?: string;
};

export function SectionHeading({ title, description, titleClassName }: Props) {
  return (
    <Stack gap={10}>
      <Title order={3} className={titleClassName}>{title}</Title>
      {description && <Description>{description}</Description>}
    </Stack>
  );
}
