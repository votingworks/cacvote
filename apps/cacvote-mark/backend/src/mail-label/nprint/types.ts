import { z } from 'zod';

export const LengthSchema = z.union([
  z.object({ type: z.literal('dot'), value: z.number().min(0) }),
  z.object({ type: z.literal('inch'), value: z.number().min(0) }),
  z.object({ type: z.literal('millimeter'), value: z.number().min(0) }),
]);
export type Length = z.infer<typeof LengthSchema>;

export const RequestSchema = z.union([
  z.object({ request: z.literal('connect'), printer: z.string().min(1) }),
  z.object({ request: z.literal('disconnect') }),
  z.object({ request: z.literal('init') }),
  z.object({ request: z.literal('lineFeed') }),
  z.object({ request: z.literal('feed'), length: LengthSchema }),
  z.object({ request: z.literal('printImage'), imagePath: z.string().min(1) }),
  z.object({ request: z.literal('printLabel'), labelPath: z.string().min(1) }),
  z.object({ request: z.literal('cut') }),
  z.object({ request: z.literal('exit') }),
]);
export type Request = z.infer<typeof RequestSchema>;

export const IncomingMessageSchema = z.object({
  replyTo: z.string().min(1),
  request: RequestSchema,
});
export type IncomingMessage = z.infer<typeof IncomingMessageSchema>;

export const ResponseSchema = z.discriminatedUnion('response', [
  z.object({ response: z.literal('ok') }),
  z.object({
    response: z.literal('error'),
    message: z.string().min(1),
    cause: z.unknown(),
  }),
]);
export type Response = z.infer<typeof ResponseSchema>;

export const StatusSchema = z.object({
  coverOpen: z.boolean(),
  noPaper: z.boolean(),
});
export type Status = z.infer<typeof StatusSchema>;

export const StatusChangeEventSchema = z.object({
  oldStatus: StatusSchema,
  newStatus: StatusSchema,
});
export type StatusChangeEvent = z.infer<typeof StatusChangeEventSchema>;

export const EventSchema = z.discriminatedUnion('eventType', [
  z.object({
    eventType: z.literal('statusChanged'),
    change: StatusChangeEventSchema,
  }),
  z.object({ eventType: z.literal('error'), message: z.string().min(1) }),
]);
export type Event = z.infer<typeof EventSchema>;

export const OutgoingMessageSchema = z.discriminatedUnion('type', [
  z.object({
    type: z.literal('response'),
    inReplyTo: z.string().min(1),
    response: ResponseSchema,
  }),
  z.object({
    type: z.literal('event'),
    event: EventSchema,
  }),
]);
export type OutgoingMessage = z.infer<typeof OutgoingMessageSchema>;
