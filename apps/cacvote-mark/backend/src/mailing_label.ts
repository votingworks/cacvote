import { Buffer } from 'buffer';
import puppeteer from 'puppeteer';
import { dirSync } from 'tmp';
import xml, { XmlAttrs, XmlObject } from 'xml';

export const SIZE_INCHES = {
  width: 4,
  height: 6,
} as const;

export const SIZE_POINTS = {
  width: SIZE_INCHES.width * 96,
  height: SIZE_INCHES.height * 96,
} as const;

function svg(attributes: XmlAttrs = {}, ...children: XmlObject[]): XmlObject {
  return { svg: [{ _attr: attributes }, ...children] };
}

function g(attributes: XmlAttrs = {}, ...children: XmlObject[]): XmlObject {
  return { g: [{ _attr: attributes }, ...children] };
}

function offset(
  { x, y }: { x: number; y: number },
  ...children: XmlObject[]
): XmlObject {
  return g({ transform: `translate(${x}, ${y})` }, ...children);
}

function rect(attributes: XmlAttrs = {}): XmlObject {
  return { rect: [{ _attr: attributes }] };
}

function line(attributes: XmlAttrs = {}): XmlObject {
  return { line: [{ _attr: attributes }] };
}

function text(value: string, attributes: XmlAttrs): XmlObject {
  return { text: [{ _attr: attributes }, value] };
}

function trackingNumberBarcode({
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

export function buildSvg({
  mailingAddress,
}: {
  mailingAddress: string;
}): string {
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

  return xml(
    svg(
      SIZE_POINTS,
      offset(
        padding,
        svg(
          inner,
          rect({
            x: 2,
            y: 2,
            width: inner.width - thickBorderSize,
            height: inner.height - thickBorderSize,
            fill: 'white',
            stroke: 'black',
            'stroke-width': thickBorderSize,
          }),
          line({
            x1: 0,
            y1: 83.52,
            x2: inner.width - thickBorderSize,
            y2: 83.52,
            stroke: 'black',
            'stroke-width': mediumBorderSize,
          }),
          line({
            x1: 0,
            y1: '4.5in',
            x2: inner.width - thickBorderSize,
            y2: '4.5in',
            stroke: 'black',
            'stroke-width': mediumBorderSize,
          }),
          offset(
            { x: 1, y: 2 },
            svg(
              { width: 63, height: 80 },
              text('E', {
                x: '50%',
                y: '73',
                'text-anchor': 'middle',
                'font-size': 92,
                'font-family': 'open-sans, sans-serif',
                fill: 'white',
                stroke: 'black',
                'stroke-width': mediumBorderSize,
              })
            )
          ),

          // Official Election Mail
          offset(
            { x: 64, y: 0 },
            line({
              x1: 0,
              y1: 0,
              x2: 0,
              y2: 83.52,
              stroke: 'black',
              'stroke-width': mediumBorderSize,
            }),
            offset(
              { x: 9.6, y: 0 },
              text('Official', {
                x: 0,
                y: 22.08,
                'dominant-baseline': 'middle',
                'font-size': 24,
                'font-family': 'open-sans, sans-serif',
                fill: 'black',
                style: 'text-transform: uppercase;',
              }),
              text('Election', {
                x: 0,
                y: 45.12,
                'dominant-baseline': 'middle',
                'font-size': 24,
                'font-family': 'open-sans, sans-serif',
                fill: 'black',
                style: 'text-transform: uppercase;',
              }),
              text('Mail', {
                x: 0,
                y: 68.16,
                'dominant-baseline': 'middle',
                'font-size': 24,
                'font-family': 'open-sans, sans-serif',
                fill: 'black',
                style: 'text-transform: uppercase;',
              })
            )
          ),

          // QR Code
          offset(
            { x: 290, y: 8 },
            svg(
              { width: 80, height: 70 },
              {
                path: {
                  _attr: {
                    d: 'm 16,16 0,16 0,16 0,16 0,16 0,16 0,16 0,16 16,0 16,0 16,0 16,0 16,0 16,0 16,0 0,-16 0,-16 0,-16 0,-16 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 -16,0 -16,0 -16,0 -16,0 z m 128,0 0,16 0,16 16,0 0,-16 16,0 0,-16 -16,0 -16,0 z m 32,16 0,16 16,0 0,-16 -16,0 z m 16,16 0,16 16,0 16,0 0,-16 0,-16 0,-16 -16,0 0,16 0,16 -16,0 z m 0,16 -16,0 -16,0 -16,0 0,16 16,0 16,0 0,16 -16,0 0,16 16,0 0,16 16,0 0,-16 16,0 0,16 -16,0 0,16 -16,0 0,16 16,0 16,0 0,16 16,0 0,-16 0,-16 0,-16 0,-16 0,-16 -16,0 0,-16 -16,0 0,-16 z m 16,112 -16,0 0,16 -16,0 0,16 0,16 16,0 0,16 0,16 -16,0 -16,0 0,-16 16,0 0,-16 -16,0 0,-16 0,-16 -16,0 0,-16 16,0 0,16 16,0 0,-16 0,-16 -16,0 -16,0 0,-16 -16,0 -16,0 -16,0 0,16 -16,0 0,16 -16,0 0,-16 16,0 0,-16 -16,0 -16,0 0,16 -16,0 0,16 0,16 0,16 16,0 0,-16 16,0 16,0 16,0 0,-16 16,0 0,-16 16,0 0,16 -16,0 0,16 16,0 0,16 -16,0 0,16 16,0 16,0 0,16 0,16 0,16 16,0 0,16 16,0 16,0 16,0 0,16 -16,0 -16,0 -16,0 0,-16 -16,0 0,16 0,16 16,0 0,16 -16,0 0,16 16,0 16,0 0,-16 16,0 0,16 16,0 16,0 16,0 16,0 0,-16 16,0 0,16 16,0 16,0 16,0 0,-16 -16,0 -16,0 0,-16 -16,0 0,-16 -16,0 0,16 -16,0 -16,0 0,16 -16,0 0,-16 16,0 0,-16 0,-16 0,-16 16,0 0,-16 -16,0 -16,0 0,-16 16,0 0,-16 0,-16 0,-16 -16,0 0,-16 z m 48,128 0,-16 -16,0 0,16 16,0 z m 32,16 16,0 16,0 0,-16 -16,0 -16,0 0,16 z m 32,-16 16,0 0,-16 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 0,-16 -16,0 0,16 -16,0 0,16 0,16 16,0 0,-16 16,0 0,16 0,16 16,0 16,0 0,16 z m -48,-80 0,-16 -16,0 -16,0 0,16 16,0 16,0 z m 16,0 16,0 0,-16 0,-16 0,-16 16,0 0,16 16,0 0,16 16,0 0,-16 0,-16 -16,0 0,-16 16,0 0,-16 -16,0 -16,0 0,16 -16,0 0,-16 -16,0 0,16 -16,0 0,16 16,0 0,16 0,16 0,16 z m -16,-48 -16,0 0,16 16,0 0,-16 z m 64,32 -16,0 0,16 16,0 0,-16 z m -224,0 0,-16 -16,0 0,16 16,0 z m -16,0 -16,0 -16,0 -16,0 0,16 16,0 16,0 16,0 0,-16 z m -64,0 -16,0 0,16 16,0 0,-16 z m 0,-48 0,-16 -16,0 0,16 16,0 z m 112,-16 16,0 0,-16 0,-16 -16,0 0,16 0,16 z m 96,-128 0,16 0,16 0,16 0,16 0,16 0,16 0,16 16,0 16,0 16,0 16,0 16,0 16,0 16,0 0,-16 0,-16 0,-16 0,-16 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 -16,0 -16,0 -16,0 -16,0 z m -208,16 16,0 16,0 16,0 16,0 16,0 0,16 0,16 0,16 0,16 0,16 -16,0 -16,0 -16,0 -16,0 -16,0 0,-16 0,-16 0,-16 0,-16 0,-16 z m 224,0 16,0 16,0 16,0 16,0 16,0 0,16 0,16 0,16 0,16 0,16 -16,0 -16,0 -16,0 -16,0 -16,0 0,-16 0,-16 0,-16 0,-16 0,-16 z m -208,16 0,16 0,16 0,16 16,0 16,0 16,0 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 z m 224,0 0,16 0,16 0,16 16,0 16,0 16,0 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 z m -32,96 0,16 16,0 0,-16 -16,0 z m -224,96 0,16 0,16 0,16 0,16 0,16 0,16 0,16 16,0 16,0 16,0 16,0 16,0 16,0 16,0 0,-16 0,-16 0,-16 0,-16 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 -16,0 -16,0 -16,0 -16,0 z m 16,16 16,0 16,0 16,0 16,0 16,0 0,16 0,16 0,16 0,16 0,16 -16,0 -16,0 -16,0 -16,0 -16,0 0,-16 0,-16 0,-16 0,-16 0,-16 z m 16,16 0,16 0,16 0,16 16,0 16,0 16,0 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 z m 288,48 0,16 16,0 0,-16 -16,0 z',
                    transform: 'scale(0.20)',
                    style: 'fill: #000000; stroke: none;',
                  },
                },
              }
            )
          ),

          // USPS Priority Mail
          offset(
            { x: 0, y: 84 },
            svg(
              { width: inner.width, height: 32 },
              text('USPS PRIORITY MAIL®', {
                x: '50%',
                y: '50%',
                'dominant-baseline': 'middle',
                'text-anchor': 'middle',
                'font-size': 20,
                'font-family': 'open-sans, sans-serif',
                fill: 'black',
                'font-weight': 600,
              }),
              line({
                x1: 0,
                y1: 28,
                x2: '100%',
                y2: 28,
                stroke: 'black',
                'stroke-width': mediumBorderSize,
              })
            )
          ),

          // Return Address
          offset(
            { x: 12, y: 84 + 35 },
            svg(
              { width: inner.width, height: 96 },
              text('Jane Doe', {
                x: 0,
                y: 0,
                'dominant-baseline': 'hanging',
                'font-size': 10,
                'font-family': 'open-sans, sans-serif',
                fill: 'black',
                style: 'text-transform: uppercase;',
              }),
              text('Example Military Base', {
                x: 0,
                y: 11,
                'dominant-baseline': 'hanging',
                'font-size': 10,
                'font-family': 'open-sans, sans-serif',
                fill: 'black',
                style: 'text-transform: uppercase;',
              }),
              text('1234 Main St', {
                x: 0,
                y: 22,
                'dominant-baseline': 'hanging',
                'font-size': 10,
                'font-family': 'open-sans, sans-serif',
                fill: 'black',
                style: 'text-transform: uppercase;',
              }),
              text('Anytown, CA 95959', {
                x: 0,
                y: 33,
                'dominant-baseline': 'hanging',
                'font-size': 10,
                'font-family': 'open-sans, sans-serif',
                fill: 'black',
                style: 'text-transform: uppercase;',
              })
            )
          ),

          // Shipping Address
          offset(
            { x: 12, y: 84 + 35 + 39 + 13 + 55 },
            svg(
              { width: inner.width, height: 96 },
              text('Ship', {
                x: 0,
                y: 0,
                'dominant-baseline': 'hanging',
                'font-size': 12,
                'font-family': 'open-sans, sans-serif',
                fill: 'black',
                style: 'text-transform: uppercase;',
              }),
              text('to:', {
                x: 0,
                y: 11,
                'dominant-baseline': 'hanging',
                'font-size': 12,
                'font-family': 'open-sans, sans-serif',
                fill: 'black',
                style: 'text-transform: uppercase;',
              }),
              ...mailingAddressLines.map((l, i) =>
                text(l, {
                  x: 55,
                  y: i * 16,
                  'dominant-baseline': 'hanging',
                  'font-size': 14,
                  'font-family': 'open-sans, sans-serif',
                  fill: 'black',
                  style: 'text-transform: uppercase;',
                })
              )
            )
          ),

          // Tracking Number
          offset(
            { x: inner.width / 20, y: 441.6 },
            svg(
              { width: inner.width - inner.width / 10, height: 96 },
              text('USPS Tracking #', {
                x: '50%',
                y: 0,
                'dominant-baseline': 'hanging',
                'text-anchor': 'middle',
                'font-size': 10,
                'font-family': 'open-sans, sans-serif',
                fill: 'black',
                style: 'text-transform: uppercase;',
              }),
              offset(
                { x: 0, y: 12 },
                trackingNumberBarcode({
                  width: inner.width - inner.width / 10,
                  height: 72,
                }),
                text('9400 1000 0000 0000 0000 00', {
                  x: '50%',
                  y: 76,
                  'dominant-baseline': 'hanging',
                  'text-anchor': 'middle',
                  'font-size': 10,
                  'font-family': 'open-sans, sans-serif',
                  fill: 'black',
                  style: 'text-transform: uppercase;',
                })
              )
            )
          )
        )
      )
    ),
    { declaration: true, indent: '  ' }
  ).toString();
}

export async function buildPdf({
  mailingAddress,
}: {
  mailingAddress: string;
}): Promise<Buffer> {
  const content = buildSvg({ mailingAddress });
  const userDataDirTemp = dirSync({ unsafeCleanup: true });
  const browser = await puppeteer.launch({
    executablePath: '/usr/bin/chromium',
    headless: 'new',
    userDataDir: userDataDirTemp.name,
  });

  const page = await browser.newPage();
  await page.setContent(content);

  const pdf = await page.pdf({
    width: `${SIZE_INCHES.width}in`,
    height: `${SIZE_INCHES.height}in`,
    printBackground: true,
  });

  await browser.close();
  userDataDirTemp.removeCallback();

  return pdf;
}
