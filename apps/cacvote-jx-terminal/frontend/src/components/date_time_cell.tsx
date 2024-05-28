import { P, TD } from '@votingworks/ui';
import { DateTime } from 'luxon';

export interface DateTimeCellProps {
  dateTime: DateTime | string;
}

export function DateTimeCell({ dateTime }: DateTimeCellProps): JSX.Element {
  const dt =
    typeof dateTime === 'string' ? DateTime.fromISO(dateTime) : dateTime;

  return (
    <TD>
      <P>{dt.toLocaleString(DateTime.DATETIME_SHORT)}</P>
    </TD>
  );
}
