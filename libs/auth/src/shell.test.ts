import { Buffer } from 'buffer';
import { Readable } from 'stream';
import { runCommand } from './shell';

test('runCommand with stdin', async () => {
  const buffer = Buffer.from('Hello, world!', 'utf-8');
  const stream = new Readable();
  stream.push(buffer);
  stream.push(null); // Signals the end of the stream

  const result = await runCommand(['cat'], {
    stdin: stream,
  });

  expect(result).toEqual(buffer);
});

test('runCommand without stdin', async () => {
  const result = await runCommand(['echo', 'Hello, world!']);

  expect(result).toEqual(Buffer.from('Hello, world!\n', 'utf-8'));
});

test('runCommand with error', async () => {
  await expect(runCommand(['cat', 'nonexistent_file'])).rejects.toThrow(
    'cat: nonexistent_file: No such file or directory'
  );
});
