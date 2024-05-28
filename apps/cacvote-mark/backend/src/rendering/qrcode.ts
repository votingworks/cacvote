import { XmlObject } from 'xml';
import { integers } from '@votingworks/basics';
import { rect, svg } from './utils';
import { qrcodegen } from './qrcodegen';

export function QrCode(data: Uint8Array): XmlObject {
  const encoded = qrcodegen.QrCode.encodeBinary(
    data,
    qrcodegen.QrCode.Ecc.MEDIUM
  );

  const constrainingSize = 70;
  const moduleSize = 16;
  const scale = constrainingSize / encoded.size / moduleSize;

  return svg(
    { width: 80, height: 70 },
    ...integers()
      .take(encoded.size)
      .flatMap((y) =>
        integers()
          .take(encoded.size)
          .map((x) =>
            rect({
              x: x * moduleSize,
              y: y * moduleSize,
              width: moduleSize,
              height: moduleSize,
              fill: encoded.getModule(x, y) ? 'black' : 'white',
              transform: `scale(${scale})`,
            })
          )
      )
  );
}
