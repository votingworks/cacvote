import { ChildProcessWithoutNullStreams, fork } from 'node:child_process';
import { join } from 'node:path';

const processes: ChildProcessWithoutNullStreams[] = [];

afterEach(() => {
  for (const process of processes) {
    process.kill();
  }
});

type Action =
  | { type: 'expect'; line: string }
  | { type: 'echo'; line: string }
  | { type: 'wait'; duration: number };

export function mockChildProcessClientTestServer(
  actions: Action[]
): ChildProcessWithoutNullStreams {
  const child = fork(
    join(__dirname, 'mock_child_process_client_test_server.js'),
    {
      stdio: 'pipe',
    }
  ) as ChildProcessWithoutNullStreams;
  child.send({ type: 'run', actions });
  child.on('message', (message) => {
    // eslint-disable-next-line no-console
    console.log('child message:', message);
  });
  processes.push(child);
  return child;
}
