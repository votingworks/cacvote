import { z } from 'zod';

export type SimpleStatus =
  | 'accepting_paper'
  | 'blank_page_interpretation'
  | 'ejecting_to_front'
  | 'ejecting_to_rear'
  | 'interpreting'
  | 'jam_cleared'
  | 'jammed'
  | 'loading_paper'
  | 'not_accepting_paper'
  | 'paper_reloaded'
  | 'presenting_ballot'
  | 'printing_ballot'
  | 'resetting_state_machine_after_jam'
  | 'resetting_state_machine_after_success'
  | 'scanning'
  | 'transition_interpretation'
  | 'waiting_for_ballot_data'
  | 'waiting_for_invalidated_ballot_confirmation';

export const SimpleStatusSchema: z.ZodSchema<SimpleStatus> = z.union([
  z.literal('accepting_paper'),
  z.literal('blank_page_interpretation'),
  z.literal('ejecting_to_front'),
  z.literal('ejecting_to_rear'),
  z.literal('interpreting'),
  z.literal('jam_cleared'),
  z.literal('jammed'),
  z.literal('loading_paper'),
  z.literal('not_accepting_paper'),
  z.literal('paper_reloaded'),
  z.literal('presenting_ballot'),
  z.literal('printing_ballot'),
  z.literal('resetting_state_machine_after_jam'),
  z.literal('resetting_state_machine_after_success'),
  z.literal('scanning'),
  z.literal('transition_interpretation'),
  z.literal('waiting_for_ballot_data'),
  z.literal('waiting_for_invalidated_ballot_confirmation'),
]);

export type SimpleServerStatus = SimpleStatus | 'no_hardware';

export const SimpleServerStatusSchema: z.ZodSchema<SimpleServerStatus> =
  z.union([SimpleStatusSchema, z.literal('no_hardware')]);