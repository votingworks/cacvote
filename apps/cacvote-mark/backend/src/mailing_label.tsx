import { Buffer } from 'buffer';
import { chromium } from 'playwright';
import React from 'react';
import ReactDomServer from 'react-dom/server';
import { writeFile } from 'fs/promises';
import { QrCode } from './rendering/qrcode';

export const SIZE_INCHES = {
  width: 4,
  height: 6,
} as const;

export const SIZE_POINTS = {
  width: SIZE_INCHES.width * 96,
  height: SIZE_INCHES.height * 96,
} as const;

export function buildSvg({
  mailingAddress,
  qrCodeData,
}: {
  mailingAddress: string;
  qrCodeData: Uint8Array;
}): JSX.Element {
  const padding = {
    x: 5.76,
    y: 12.48,
  } as const;
  const inner = {
    width: SIZE_POINTS.width - padding.x * 2,
    height: SIZE_POINTS.height - padding.y * 2,
  } as const;
  const thickBorderSize = 4;
  const mediumBorderSize = 3;

  const mailingAddressLines = mailingAddress.split('\n').map((l) => l.trim());

  return (
    <svg width={SIZE_POINTS.width} height={SIZE_POINTS.height}>
      <g transform={`translate(${padding.x}, ${padding.y})`}>
        <svg width={inner.width} height={inner.height}>
          <rect
            x={2}
            y={2}
            width={inner.width - thickBorderSize}
            height={inner.height - thickBorderSize}
            fill="white"
            stroke="black"
            strokeWidth={thickBorderSize}
          />
          <line
            x1={0}
            y1={83.52}
            x2={inner.width - thickBorderSize}
            y2={83.52}
            stroke="black"
            strokeWidth={mediumBorderSize}
          />
          <line
            x1={0}
            y1={'4.5in'}
            x2={inner.width - thickBorderSize}
            y2={'4.5in'}
            stroke="black"
            strokeWidth={mediumBorderSize}
          />
        </svg>
        <g transform="translate(1, 2)">
          <svg width={63} height={80}>
            <text
              x="50%"
              y={73}
              textAnchor="middle"
              fontSize="92"
              fontFamily="open-sans, sans-serif"
              fill="white"
              stroke="black"
              strokeWidth={mediumBorderSize}
            >
              E
            </text>
          </svg>
        </g>
        <g transform="translate(64, 0)">
          <line
            x1={0}
            y1={0}
            x2={0}
            y2={83.52}
            stroke="black"
            strokeWidth={mediumBorderSize}
          />
          <g transform="translate(9.6, 0)">
            <text
              x={0}
              y={22.08}
              dominantBaseline="middle"
              fontSize={24}
              fontFamily="open-sans, sans-serif"
              fill="black"
            >
              Official
            </text>
            <text
              x={0}
              y={45.12}
              dominantBaseline="middle"
              fontSize={24}
              fontFamily="open-sans, sans-serif"
              fill="black"
            >
              Election
            </text>
            <text
              x={0}
              y={68.16}
              dominantBaseline="middle"
              fontSize={24}
              fontFamily="open-sans, sans-serif"
              fill="black"
            >
              Mail
            </text>
          </g>
        </g>

        {/* QR Code */}
        <g transform="translate(290, 8)">
          <QrCode data={qrCodeData} />
        </g>

        {/* USPS Priority Mail */}
        <g transform="translate(0, 84)">
          <svg width={inner.width} height={32}>
            <text
              x="50%"
              y="50%"
              dominantBaseline="middle"
              textAnchor="middle"
              fontSize={20}
              fontFamily="open-sans, sans-serif"
              fill="black"
              fontWeight={600}
            >
              USPS PRIORITY MAIL®
            </text>
            <line
              x1={0}
              y1={28}
              x2="100%"
              y2={28}
              stroke="black"
              strokeWidth={mediumBorderSize}
            />
          </svg>
        </g>

        {/* Shipping Address */}
        <g transform={`translate(12, ${84 + 35 + 39 + 13 + 55})`}>
          <svg width={inner.width} height={96}>
            <text
              x={0}
              y={0}
              dominantBaseline="hanging"
              fontSize={12}
              fontFamily="open-sans, sans-serif"
              fill="black"
            >
              Ship
            </text>
            <text
              x={0}
              y={11}
              dominantBaseline="hanging"
              fontSize={12}
              fontFamily="open-sans, sans-serif"
              fill="black"
            >
              to:
            </text>
            {mailingAddressLines.map((l, i) => (
              <text
                key={`line-${i}`}
                x={55}
                y={i * 16}
                dominantBaseline="hanging"
                fontSize={14}
                fontFamily="open-sans, sans-serif"
                fill="black"
              >
                {l}
              </text>
            ))}
          </svg>
        </g>
      </g>
    </svg>
  );
}

async function renderToPdf(document: JSX.Element): Promise<Buffer> {
  const documentHtml = ReactDomServer.renderToString(document);

  const browser = await chromium.launch({
    // font hinting (https://fonts.google.com/knowledge/glossary/hinting)
    // is on by default, but causes fonts to render more awkwardly at higher
    // resolutions, so we disable it
    args: ['--font-render-hinting=none'],
  });
  const context = await browser.newContext();
  const page = await context.newPage();

  await page.setContent(documentHtml);
  const pdfBuffer = await page.pdf({
    width: `${SIZE_INCHES.width}in`,
    height: `${SIZE_INCHES.height}in`,
    printBackground: true, // necessary to render shaded backgrounds
  });
  await context.close();
  await browser.close();

  return pdfBuffer;
}

export async function buildPdf({
  mailingAddress,
  qrCodeData,
}: {
  mailingAddress: string;
  qrCodeData: Uint8Array;
}): Promise<Buffer> {
  return await renderToPdf(buildSvg({ mailingAddress, qrCodeData }));
}

if (require.main === module) {
  void (async () => {
    const qrCodeData = Buffer.from('hello, world');
    const mailingAddress = `VotingWorks\n123 Main St\nAnytown, USA 12345`;

    const pdfBuffer = await buildPdf({ mailingAddress, qrCodeData });
    await writeFile('mailing_label.pdf', pdfBuffer);
  })();
}
