import { z } from 'zod';

export interface PrintOptions extends KioskBrowser.PrintOptions {
  deviceName?: string;
  sides: KioskBrowser.PrintSides;
  raw?: { [key: string]: string };
}
export interface Printer {
  print(options: PrintOptions): Promise<void>;
}

export type PrecinctReportDestination =
  | 'laser-printer'
  | 'thermal-sheet-printer';

export const PrecinctReportDestinationSchema: z.ZodSchema<PrecinctReportDestination> =
  z.union([z.literal('laser-printer'), z.literal('thermal-sheet-printer')]);
