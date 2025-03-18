import { Deferred, assert, deferred, deferredQueue } from '@votingworks/basics';
import { ChildProcessWithoutNullStreams } from 'child_process';

export class ChildProcessClient<
  Request extends { replyTo: string },
  Response extends { inReplyTo: string },
  Event,
> {
  private child?: ChildProcessWithoutNullStreams;
  private readonly deferredEvents = deferredQueue<Event>();
  private readonly deferredResponses = new Map<string, Deferred<Response>>();

  // eslint-disable-next-line vx/gts-parameter-properties
  constructor(child: ChildProcessWithoutNullStreams) {
    child.stdout.setEncoding('utf-8');

    let buffer = '';
    child.stdout.on('data', (data) => {
      buffer += data;
      // eslint-disable-next-line no-constant-condition
      while (true) {
        const newlineIndex = buffer.indexOf('\n');
        if (newlineIndex === -1) break;
        const message = buffer.slice(0, newlineIndex);
        buffer = buffer.slice(newlineIndex + 1);
        const parsedMessage = JSON.parse(message);

        if ('inReplyTo' in parsedMessage) {
          const deferredResponse = this.deferredResponses.get(
            parsedMessage.inReplyTo
          );
          if (deferredResponse) {
            deferredResponse.resolve(parsedMessage);
            this.deferredResponses.delete(parsedMessage.inReplyTo);
          }
        } else {
          this.deferredEvents.resolve(parsedMessage);
        }
      }
    });

    child.stderr.pipe(process.stderr);

    child.once('exit', () => {
      this.child = undefined;
    });

    this.child = child;
  }

  async send(request: Request): Promise<Response> {
    const deferredResponse = deferred<Response>();
    assert(this.child, 'Child process is not running');
    assert(
      !this.deferredResponses.has(request.replyTo),
      `Duplicate replyTo: ${request.replyTo}`
    );

    this.deferredResponses.set(request.replyTo, deferredResponse);
    this.child.stdin.write(`${JSON.stringify(request)}\n`);
    return await deferredResponse.promise;
  }

  async close(): Promise<void> {
    if (!this.child) return;
    const { child } = this;
    this.child = undefined;

    child.kill();
    await new Promise((resolve) => {
      child.once('close', resolve);
    });
  }

  isConnected(): boolean {
    return !!this.child;
  }

  async *events(): AsyncIterableIterator<Event> {
    while (true) {
      yield await this.deferredEvents.get();
    }
  }
}
