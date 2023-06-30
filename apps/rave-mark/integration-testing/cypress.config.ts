/* eslint-disable no-console */
import { resolveWorkspace } from '@votingworks/rave-mark-backend';
import { Id, safeParseInt } from '@votingworks/types';
import { defineConfig } from 'cypress';

const basePort = safeParseInt(process.env['BASE_PORT']).ok() ?? 3000;
const port = safeParseInt(process.env['PORT']).ok() ?? basePort;

function createUniqueCommonAccessCardId(): Id {
  const tenRandomDigits = Math.floor(Math.random() * 1e10).toString();
  return `test-${tenRandomDigits.toString().padStart(10, '0')}`;
}

export default defineConfig({
  e2e: {
    baseUrl: `http://localhost:${port}`,

    setupNodeEvents(on) {
      on('task', {
        createVoter(options: Cypress.CreateVoterOptions = {}) {
          const commonAccessCardId = createUniqueCommonAccessCardId();
          const workspace = resolveWorkspace();
          console.log('Found workspace at path:', workspace.path);
          console.log('Creating voter with CAC ID:', commonAccessCardId);
          const voterInfo =
            workspace.store.getOrCreateVoterInfo(commonAccessCardId);
          workspace.store.setVoterIsAdmin(
            voterInfo.id,
            options.isAdmin ?? false
          );

          if (options.registration) {
            const registrationId = workspace.store.createVoterRegistration({
              commonAccessCardId,
              givenName: options.registration.givenName ?? 'Rebecca',
              familyName: options.registration.familyName ?? 'Welton',
              addressLine1: options.registration.addressLine1 ?? '123 Main St',
              addressLine2: options.registration.addressLine2,
              city: options.registration.city ?? 'Anytown',
              state: options.registration.state ?? 'CA',
              postalCode: options.registration.postalCode ?? '95959',
              stateId: options.registration.stateId ?? 'B2201793',
            });

            if (options.registration.electionData) {
              const electionId = workspace.store.createElectionDefinition(
                options.registration.electionData.toString()
              );

              workspace.store.setVoterRegistrationElection(
                registrationId,
                electionId
              );
            }
          }

          return commonAccessCardId;
        },
      });
    },
  },

  viewportWidth: 1920,
  viewportHeight: 1080,
});
