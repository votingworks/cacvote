import { Buffer } from 'buffer';
import { readFile } from 'fs/promises';
import { join } from 'path';
import { chromium } from 'playwright';
import React from 'react';
import ReactDomServer from 'react-dom/server';
import { QrCode } from './qrcode';

export const SIZE_INCHES = {
  width: 4,
  height: 6,
} as const;

export function build({
  mailingAddress,
  qrCodeData,
}: {
  mailingAddress: string;
  qrCodeData: Uint8Array;
}): JSX.Element {
  const mailingAddressLines = mailingAddress.split('\n').map((l) => l.trim());

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        border: '4px solid black',
        width: `${SIZE_INCHES.width}in`,
        height: `${SIZE_INCHES.height}in`,
        fontFamily: 'Arial, sans-serif',
      }}
    >
      <div
        id="header"
        style={{
          display: 'flex',
          flexDirection: 'row',
          alignItems: 'center',
          width: '100%',
        }}
      >
        <div
          style={{
            fontSize: '6em',
            WebkitTextStroke: '2px black',
            color: 'white',
            flexGrow: 0,
            padding: '0 10px',
            fontWeight: 'bold',
          }}
        >
          E
        </div>
        <div style={{ fontSize: '1.3em' }}>
          Official
          <br />
          Election
          <br />
          Mail
        </div>
      </div>
      <div
        id="content"
        style={{
          display: 'flex',
          flexDirection: 'row',
          flexGrow: 1,
          alignItems: 'center',
          alignContent: 'center',
          borderTop: '4px solid black',
          borderBottom: '4px solid black',
          width: '100%',
          padding: '3em',
          gap: '2em',
        }}
      >
        <div>
          Ship
          <br />
          to:
        </div>
        <div style={{ flexDirection: 'column', fontSize: '1.4em' }}>
          {mailingAddressLines.map((line, index) => (
            <div key={index}>{line}</div>
          ))}
        </div>
      </div>
      <div
        id="footer"
        style={{
          display: 'flex',
          width: '100%',
          justifyContent: 'end',
          padding: '1em',
        }}
      >
        <QrCode data={qrCodeData} />
      </div>
    </div>
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
  return await renderToPdf(
    <body>
      <style>
        {await readFile(join(__dirname, 'modern-normalize.css'), 'utf8')}
      </style>
      {build({ mailingAddress, qrCodeData })}
    </body>
  );
}
