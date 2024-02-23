// This file sets up React's proxy in development mode.
//
// Currently, non-native Node languages (e.g. typescript) are explicitly not supported:
// https://facebook.github.io/create-react-app/docs/proxying-api-requests-in-development#configuring-the-proxy-manually
//
/* eslint-disable */
/* istanbul ignore file */

const { createProxyMiddleware: proxy } = require('http-proxy-middleware');

/**
 * @param {import('connect').Server} app
 * @param {number=} basePort
 */
module.exports = function (app, basePort = 3000) {
  app.use(proxy('/api', { target: `http://localhost:${basePort + 2}/` }));
  app.use(proxy('/dock', { target: `http://localhost:${basePort + 2}/` }));
};
