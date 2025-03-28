import { integers } from '@votingworks/basics';
import React from 'react';
import { qrcodegen } from './qrcodegen';

export function QrCode({ data }: { data: Uint8Array }): JSX.Element {
  const encoded = qrcodegen.QrCode.encodeBinary(
    data,
    qrcodegen.QrCode.Ecc.MEDIUM
  );

  const constrainingSize = 87.5;
  const moduleSize = 20;
  const scale = constrainingSize / encoded.size / moduleSize;

  return (
    <svg width={100} height={87.5}>
      {[
        ...integers()
          .take(encoded.size)
          .flatMap((y) =>
            integers()
              .take(encoded.size)
              .map((x) => (
                <rect
                  key={`${x},${y}`}
                  x={x * moduleSize}
                  y={y * moduleSize}
                  width={moduleSize}
                  height={moduleSize}
                  fill={encoded.getModule(x, y) ? 'black' : 'white'}
                  transform={`scale(${scale})`}
                />
              ))
          ),
      ]}
    </svg>
  );
}
