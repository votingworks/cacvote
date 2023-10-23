export interface PrintOptions extends KioskBrowser.PrintOptions {
  deviceName?: string;
  sides: KioskBrowser.PrintSides;
  raw?: { [key: string]: string };
}
export interface Printer {
  print(options: PrintOptions): Promise<void>;
}
