import { P, TD } from '@votingworks/ui';

export interface VoterInfoCellProps {
  displayName: string;
  commonAccessCardId: string;
}

export function VoterInfoCell({
  displayName,
  commonAccessCardId,
}: VoterInfoCellProps): JSX.Element {
  return (
    <TD>
      <P>{displayName}</P>
      <P>
        <em>CAC:</em> {commonAccessCardId}
      </P>
    </TD>
  );
}
