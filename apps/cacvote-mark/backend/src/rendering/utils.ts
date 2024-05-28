import { XmlAttrs, XmlObject } from 'xml';

export function svg(
  attributes: XmlAttrs = {},
  ...children: XmlObject[]
): XmlObject {
  return { svg: [{ _attr: attributes }, ...children] };
}

export function g(
  attributes: XmlAttrs = {},
  ...children: XmlObject[]
): XmlObject {
  return { g: [{ _attr: attributes }, ...children] };
}

export function offset(
  { x, y }: { x: number; y: number },
  ...children: XmlObject[]
): XmlObject {
  return g({ transform: `translate(${x}, ${y})` }, ...children);
}

export function rect(attributes: XmlAttrs = {}): XmlObject {
  return { rect: [{ _attr: attributes }] };
}

export function line(attributes: XmlAttrs = {}): XmlObject {
  return { line: [{ _attr: attributes }] };
}

export function text(value: string, attributes: XmlAttrs): XmlObject {
  return { text: [{ _attr: attributes }, value] };
}
