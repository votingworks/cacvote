/* @ts-check */

// eslint-disable-next-line @typescript-eslint/no-var-requires
const { deferredQueue } = require('@votingworks/basics');

let buffer = '';
const lines = deferredQueue();
process.stdin.on('data', (data) => {
  buffer += data.toString();
  // eslint-disable-next-line no-constant-condition
  while (true) {
    const newlineIndex = buffer.indexOf('\n');
    if (newlineIndex === -1) break;
    const line = buffer.slice(0, newlineIndex);
    buffer = buffer.slice(newlineIndex + 1);
    lines.resolve(line);
  }
});

async function runActions(actions) {
  for (const action of actions) {
    switch (action.type) {
      case 'expect': {
        const line = await lines.get();
        if (line !== action.line) {
          process.stderr.write(
            `Expected "${action.line}", but got "${line}"\n`
          );
          process.exit(1);
        }
        break;
      }

      case 'echo': {
        process.stdout.write(`${action.line}\n`);
        break;
      }

      case 'wait': {
        await new Promise((resolve) => {
          setTimeout(resolve, action.duration);
        });
        break;
      }

      default: {
        process.stderr.write(`Unknown action type: ${action.type}\n`);
        break;
      }
    }
  }
}

process.addListener('message', async (message) => {
  if (message?.type === 'run') {
    await runActions(message.actions);
  } else if (message?.type === 'exit') {
    process.exit(0);
  } else {
    process.stderr.write(`Unknown message type: ${message}\n`);
  }
});
