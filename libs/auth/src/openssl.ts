import { Buffer } from 'buffer';
import fs from 'fs/promises';
import path from 'path';
import { Stream } from 'stream';
import { FileResult, fileSync } from 'tmp';

import { runCommand } from './shell';

/**
 * The path to the OpenSSL config file
 */
export const OPENSSL_CONFIG_FILE_PATH = path.join(
  __dirname,
  '../certs/openssl.cnf'
);

/**
 * The static header for a public key in DER format
 */
export const PUBLIC_KEY_IN_DER_FORMAT_HEADER = Buffer.from([
  0x30, 0x59, 0x30, 0x13, 0x06, 0x07, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x02, 0x01,
  0x06, 0x08, 0x2a, 0x86, 0x48, 0xce, 0x3d, 0x03, 0x01, 0x07, 0x03, 0x42, 0x00,
]);

type OpensslParam = string | Buffer;

/**
 * A convenience function for OpenSSL shell commands. For file params, accepts Buffers containing
 * the file's contents. Writes these Buffers to temporary files and deletes these files after
 * completion of the OpenSSL command.
 *
 * The returned promise resolves if the shell command's exit status is 0 and rejects otherwise
 * (OpenSSL cert and signature verification commands return non-zero exit statuses when
 * verification fails). The promise also rejects if cleanup of temporary files fails.
 *
 * Sample usage:
 * await openssl(['verify', '-CAfile', '/path/to/cert/authority/cert.pem', certToVerifyAsBuffer]);
 */
export async function openssl(
  params: OpensslParam[],
  { stdin }: { stdin?: Stream } = {}
): Promise<Buffer> {
  const processedParams: string[] = [];
  const tempFileResults: FileResult[] = [];
  for (const param of params) {
    if (Buffer.isBuffer(param)) {
      const tempFile = fileSync();
      await fs.writeFile(tempFile.name, param);
      processedParams.push(tempFile.name);
      tempFileResults.push(tempFile);
    } else {
      processedParams.push(param);
    }
  }

  let stdout: Buffer;
  try {
    stdout = await runCommand(['openssl', ...processedParams], { stdin });
  } finally {
    await Promise.all(
      tempFileResults.map((tempFile) => tempFile.removeCallback())
    );
  }
  return stdout;
}

/**
 * An alias for clarity
 */
type FilePathOrBuffer = string | Buffer;

/**
 * Converts a cert in DER format to PEM format
 */
export function certDerToPem(cert: FilePathOrBuffer): Promise<Buffer> {
  return openssl(['x509', '-inform', 'DER', '-outform', 'PEM', '-in', cert]);
}

/**
 * Extracts a public key (in PEM format) from a cert (also in PEM format)
 */
export function extractPublicKeyFromCert(
  cert: FilePathOrBuffer
): Promise<Buffer> {
  return openssl(['x509', '-noout', '-pubkey', '-in', cert]);
}

/**
 * Verifies a message signature against an original message using a public key (in PEM format).
 * Throws an error if signature verification fails.
 */
export async function verifySignature({
  message,
  messageSignature,
  publicKey,
}: {
  message: FilePathOrBuffer | Stream;
  messageSignature: FilePathOrBuffer;
  publicKey: FilePathOrBuffer;
}): Promise<void> {
  const params = [
    'dgst',
    '-sha256',
    '-verify',
    publicKey,
    '-signature',
    messageSignature,
  ];
  if (typeof message === 'string' || Buffer.isBuffer(message)) {
    await openssl([...params, message]);
    return;
  }
  await openssl(params, { stdin: message });
}
