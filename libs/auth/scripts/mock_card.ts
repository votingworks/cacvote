import {
  DateWithoutTime,
  Optional,
  assert,
  extractErrorMessage,
  throwIllegalValue,
} from '@votingworks/basics';
import { readFile } from '@votingworks/fs';
import { ElectionId, ElectionKey, safeParseJson } from '@votingworks/types';
import yargs from 'yargs/yargs';
import { DEV_JURISDICTION } from '../src/jurisdictions';
import { mockCard } from '../src/mock_file_card';

const CARD_TYPES = [
  'system-administrator',
  'election-manager',
  'poll-worker',
  'poll-worker-with-pin',
  'unprogrammed',
  'no-card',
] as const;
type CardType = (typeof CARD_TYPES)[number];

interface MockCardInput {
  cardType: CardType;
  electionKey?: ElectionKey;
}

async function parseCommandLineArgs(): Promise<MockCardInput> {
  const argParser = yargs()
    .options({
      'card-type': {
        description: 'The type of card to mock',
        type: 'string',
        choices: CARD_TYPES,
      },
      'election-definition': {
        description:
          'The election definition to use for an election manager or poll worker card',
        type: 'string',
      },
    })
    .hide('help')
    .version(false)
    .example('$ mock-card --help', '')
    .example('$ mock-card --card-type system-administrator', '')
    .example(
      '$ mock-card --card-type election-manager \\\n' +
        '--election-definition path/to/election.json',
      ''
    )
    .example(
      '$ mock-card --card-type poll-worker \\\n' +
        '--election-definition path/to/election.json',
      ''
    )
    .example(
      '$ mock-card --card-type poll-worker-with-pin \\\n' +
        '--election-definition path/to/election.json',
      ''
    )
    .example('$ mock-card --card-type unprogrammed', '')
    .example('$ mock-card --card-type no-card', '')
    .strict();

  const helpMessage = await argParser.getHelp();
  argParser.fail((errorMessage: string) => {
    throw new Error(`${errorMessage}\n\n${helpMessage}`);
  });

  const args = argParser.parse(process.argv.slice(2)) as {
    cardType?: CardType;
    electionDefinition?: string;
    help?: boolean;
  };

  if (args.help || process.argv.length === 2) {
    console.log(helpMessage);
    process.exit(0);
  }

  if (!args.cardType) {
    throw new Error(`Must specify card type\n\n${helpMessage}`);
  }

  let electionKey: Optional<ElectionKey>;
  if (['election-manager', 'poll-worker'].includes(args.cardType)) {
    if (!args.electionDefinition) {
      throw new Error(
        `Must specify election definition for election manager and poll worker cards\n\n${helpMessage}`
      );
    }
    const readElectionResult = safeParseJson(
      (await readFile(args.electionDefinition, { maxSize: 1024 * 1024 }))
        .unsafeUnwrap()
        .toString('utf-8')
    );
    if (readElectionResult.isErr()) {
      throw new Error(
        `${args.electionDefinition} isn't a valid election definition`
      );
    }
    const election = readElectionResult.ok() as {
      id: string;
      date: string;
    };
    electionKey = {
      id: election.id as ElectionId,
      date: new DateWithoutTime(election.date),
    };
  }

  return {
    cardType: args.cardType,
    electionKey,
  };
}

function mockCardWrapper({ cardType, electionKey }: MockCardInput) {
  switch (cardType) {
    case 'system-administrator': {
      mockCard({
        cardStatus: {
          status: 'ready',
          cardDetails: {
            user: {
              role: 'system_administrator',
              jurisdiction: DEV_JURISDICTION,
            },
          },
        },
        pin: '000000',
      });
      break;
    }
    case 'election-manager': {
      assert(electionKey !== undefined);
      mockCard({
        cardStatus: {
          status: 'ready',
          cardDetails: {
            user: {
              role: 'election_manager',
              jurisdiction: DEV_JURISDICTION,
              electionKey,
            },
          },
        },
        pin: '000000',
      });
      break;
    }
    case 'poll-worker': {
      assert(electionKey !== undefined);
      mockCard({
        cardStatus: {
          status: 'ready',
          cardDetails: {
            user: {
              role: 'poll_worker',
              jurisdiction: DEV_JURISDICTION,
              electionKey,
            },
            hasPin: false,
          },
        },
      });
      break;
    }
    case 'poll-worker-with-pin': {
      assert(electionKey !== undefined);
      mockCard({
        cardStatus: {
          status: 'ready',
          cardDetails: {
            user: {
              role: 'poll_worker',
              jurisdiction: DEV_JURISDICTION,
              electionKey,
            },
            hasPin: true,
          },
        },
        pin: '000000',
      });
      break;
    }
    case 'unprogrammed': {
      mockCard({
        cardStatus: {
          status: 'ready',
          cardDetails: undefined,
        },
      });
      break;
    }
    case 'no-card': {
      mockCard({
        cardStatus: {
          status: 'no_card',
        },
      });
      break;
    }
    /* istanbul ignore next: Compile-time check for completeness */
    default: {
      throwIllegalValue(cardType);
    }
  }
}

/**
 * A script for mocking cards during local development. Run with --help for further guidance.
 */
export async function main(): Promise<void> {
  try {
    mockCardWrapper(await parseCommandLineArgs());
  } catch (error) {
    console.error(`❌ ${extractErrorMessage(error)}`);
    process.exit(1);
  }
}
