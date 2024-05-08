import { XmlObject } from 'xml';
import { g, line } from './utils';

export function randomTrackingNumberBarcode({
  width,
  height,
}: {
  width: number;
  height: number;
}): XmlObject {
  const lines: XmlObject[] = [];

  let bias = 0;

  for (let i = 0; i < width * 2; i += 1) {
    if (Math.random() < 0.5 - bias) {
      lines.push(
        line({
          x1: i,
          y1: 0,
          x2: i,
          y2: height,
          stroke: 'black',
          'stroke-width': 1,
        })
      );
      bias += 0.1;
    } else {
      bias -= 0.1;
    }
  }

  return g({}, ...lines);
}
