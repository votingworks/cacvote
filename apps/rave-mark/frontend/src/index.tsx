import './polyfills';
import React from 'react';
import ReactDom from 'react-dom';
import { DevDock } from '@votingworks/dev-dock-frontend';
import { App } from './app';

ReactDom.render(
  <React.Fragment>
    <App />
    <DevDock />
  </React.Fragment>,
  document.getElementById('root') as HTMLElement
);
