import { Buffer } from 'buffer';
import { sleep } from '@votingworks/basics';
import { CARD_CERT, JavaCard } from './java_card';
import { Card } from './card';

/**
 * Waits for a card to have a ready status
 */
async function waitForReadyCardStatus(
  card: Card,
  waitTimeSeconds = 3
): Promise<void> {
  let cardStatus = await card.getCardStatus();
  let remainingWaitTimeSeconds = waitTimeSeconds;
  while (cardStatus.status !== 'ready' && remainingWaitTimeSeconds > 0) {
    await sleep(1000);
    cardStatus = await card.getCardStatus();
    remainingWaitTimeSeconds -= 1;
  }
  if (cardStatus.status !== 'ready') {
    throw new Error(`Card status not "ready" after ${waitTimeSeconds} seconds`);
  }
}

test.skip('live card', async () => {
  Object.assign(process.env, { VX_MACHINE_TYPE: 'mark' });
  const card = new JavaCard();
  await waitForReadyCardStatus(card);
  await card.getCertificate({ objectId: CARD_CERT.OBJECT_ID });
  await card.checkPin('77777777');
  await card.getCertificate({ objectId: CARD_CERT.OBJECT_ID });
  await card.generateSignature(Buffer.from('hello world'), {
    privateKeyId: CARD_CERT.PRIVATE_KEY_ID,
    pin: '77777777',
  });
  await card.disconnect();
});
