import { useDevDockRouter } from '@votingworks/dev-dock-backend';
import * as grout from '@votingworks/grout';
import express, { Application } from 'express';

function buildApi() {
  return grout.createApi({});
}

export type Api = ReturnType<typeof buildApi>;

export function buildApp(): Application {
  const app: Application = express();
  const api = buildApi();
  app.use('/api', grout.buildRouter(api, express));
  useDevDockRouter(app, express);
  return app;
}
