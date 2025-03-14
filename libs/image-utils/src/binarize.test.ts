import {
  assertBinaryImageDatasEqual,
  makeBinaryImageData,
} from '../test/utils';
import { binarize } from './binarize';

test.each([1, 4] as const)(
  'already-binarized image with %s channels',
  (channelCount) => {
    const allBackground = makeBinaryImageData(
      `
        ......
        ......
        ......
      `,
      channelCount
    );

    assertBinaryImageDatasEqual(binarize(allBackground), allBackground);

    const allForeground = makeBinaryImageData(
      `
        ######
        ######
        ######
      `,
      channelCount
    );

    assertBinaryImageDatasEqual(binarize(allForeground), allForeground);
  }
);
