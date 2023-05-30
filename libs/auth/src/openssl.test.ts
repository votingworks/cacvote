import {
  FakeChildProcess as MockChildProcess,
  mockOf,
  fakeChildProcess as newMockChildProcess,
} from '@votingworks/test-utils';
import { Buffer } from 'buffer';
import { spawn } from 'child_process';
import fs from 'fs/promises';
import { Writable } from 'stream';
import { fileSync } from 'tmp';

import { openssl } from './openssl';

jest.mock('child_process');
jest.mock('tmp');

let mockChildProcess: MockChildProcess;
let nextTempFileName = 0;
const tempFileRemoveCallbacks: jest.Mock[] = [];

beforeEach(() => {
  mockChildProcess = newMockChildProcess();
  mockOf(spawn).mockImplementation(() => mockChildProcess);

  nextTempFileName = 0;
  mockOf(fileSync).mockImplementation(() => {
    nextTempFileName += 1;
    const removeCallback = jest.fn();
    tempFileRemoveCallbacks.push(removeCallback);
    return {
      fd: nextTempFileName,
      name: `/tmp/openssl/${nextTempFileName}`,
      removeCallback,
    };
  });
  jest.spyOn(fs, 'writeFile').mockResolvedValue();

  jest.spyOn(process.stdin, 'pipe').mockImplementation(() => new Writable());
});

const fileBuffers = [
  Buffer.from('file1Contents', 'utf-8'),
  Buffer.from('file2contents', 'utf-8'),
] as const;
const tempFilePaths = [
  '/tmp/openssl/1',
  '/tmp/openssl/2',
  '/tmp/openssl/3',
] as const;
const responseChunks = [
  Buffer.from('Hey!', 'utf-8'),
  Buffer.from(' ', 'utf-8'),
  Buffer.from('How is it going?', 'utf-8'),
] as const;
const errorChunks = [
  Buffer.from('Uh ', 'utf-8'),
  Buffer.from('oh!', 'utf-8'),
] as const;
const successExitCode = 0;
const errorExitCode = 1;

test('openssl', async () => {
  setTimeout(() => {
    responseChunks.forEach((responseChunk) => {
      mockChildProcess.stdout.emit('data', responseChunk);
    });
    mockChildProcess.stderr.emit('data', Buffer.from('Some warning', 'utf-8'));
    mockChildProcess.emit('close', successExitCode);
  });

  const response = await openssl([
    'some',
    fileBuffers[0],
    'command',
    fileBuffers[1],
  ]);
  expect(response.toString('utf-8')).toEqual('Hey! How is it going?');

  expect(fileSync).toHaveBeenCalledTimes(2);
  expect(spawn).toHaveBeenCalledTimes(1);
  expect(spawn).toHaveBeenNthCalledWith(1, 'openssl', [
    'some',
    tempFilePaths[0],
    'command',
    tempFilePaths[1],
  ]);
  for (const tempFileRemoveCallback of tempFileRemoveCallbacks) {
    expect(tempFileRemoveCallback).toHaveBeenCalledTimes(1);
  }
});

test('openssl - no Buffer params', async () => {
  setTimeout(() => {
    for (const responseChunk of responseChunks) {
      mockChildProcess.stdout.emit('data', responseChunk);
    }
    mockChildProcess.emit('close', successExitCode);
  });

  const response = await openssl(['some', 'command']);
  expect(response.toString('utf-8')).toEqual('Hey! How is it going?');

  expect(fileSync).toHaveBeenCalledTimes(0);
  expect(spawn).toHaveBeenCalledTimes(1);
  expect(spawn).toHaveBeenNthCalledWith(1, 'openssl', ['some', 'command']);
});

test('openssl - no standard output', async () => {
  setTimeout(() => {
    mockChildProcess.emit('close', successExitCode);
  });

  const response = await openssl(['some', 'command']);
  expect(response).toEqual(Buffer.from([]));
});

test('openssl - error creating temporary file', async () => {
  mockOf(fileSync).mockImplementationOnce(() => {
    throw new Error('Whoa!');
  });

  await expect(openssl([fileBuffers[0]])).rejects.toThrow('Whoa!');
});

test('openssl - error writing temp file', async () => {
  mockOf(fs.writeFile).mockImplementationOnce(() =>
    Promise.reject(new Error('Whoa!'))
  );

  await expect(openssl([fileBuffers[0]])).rejects.toThrow('Whoa!');
});

test('openssl - error cleaning up temp files', async () => {
  setTimeout(() => {
    for (const tempFileRemoveCallback of tempFileRemoveCallbacks) {
      tempFileRemoveCallback.mockRejectedValueOnce(new Error('Whoa!'));
    }
    mockChildProcess.emit('close', successExitCode);
  });

  await expect(openssl([fileBuffers[0]])).rejects.toThrow('Whoa!');
});

test('openssl - process exits with an error code', async () => {
  setTimeout(() => {
    for (const errorChunk of errorChunks) {
      mockChildProcess.stderr.emit('data', errorChunk);
    }
    mockChildProcess.emit('close', errorExitCode);
  });

  await expect(openssl([fileBuffers[0]])).rejects.toThrow('Uh oh!');
});

test('openssl - provides both stderr and stdout on error', async () => {
  setTimeout(() => {
    for (const errorChunk of errorChunks) {
      mockChildProcess.stderr.emit('data', errorChunk);
    }
    for (const responseChunk of responseChunks) {
      mockChildProcess.stdout.emit('data', responseChunk);
    }
    mockChildProcess.emit('close', errorExitCode);
  });

  await expect(openssl([fileBuffers[0]])).rejects.toThrow(
    'Uh oh!\nHey! How is it going?'
  );
});
