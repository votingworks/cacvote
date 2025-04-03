import React from 'react';
import { qrcodegen } from './qrcodegen';

/**
 * The Nippon thermal label printer can print 8 dots per millimeter, and
 * https://www.qrcode.com/en/howto/cell.html recommends using 0.5mm for that density.
 */
const MIN_MODULE_SIZE_MM = 0.5;

export function QrCode({ data }: { data: Uint8Array }): JSX.Element {
  const encoded = qrcodegen.QrCode.encodeBinary(
    data,
    qrcodegen.QrCode.Ecc.MEDIUM
  );

  const goOrigin = 'M 0 0 ';
  const drawSquare = 'h 1 v 1 h -1 z ';
  const goNextColumn = 'm 1 0 ';
  const goNextRow = `m ${-encoded.size} 1 `;

  let d = goOrigin;
  for (let y = 0; y < encoded.size; y += 1) {
    for (let x = 0; x < encoded.size; x += 1) {
      if (encoded.getModule(x, y)) {
        d += drawSquare;
      }
      d += goNextColumn;
    }
    d += goNextRow;
  }

  return (
    <svg
      // set the viewBox to the module size of the QR code
      viewBox={`0 0 ${encoded.size} ${encoded.size}`}
      // set the width and height to the desired physical size of the QR code in millimeters
      width={`${encoded.size * MIN_MODULE_SIZE_MM}mm`}
      height={`${encoded.size * MIN_MODULE_SIZE_MM}mm`}
    >
      <path d={d} fill="black" />
    </svg>
  );
}
