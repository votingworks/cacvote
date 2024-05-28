import { assert } from '@votingworks/basics';
import { EventEmitter } from 'node:events';

export interface Mess {
  install(): void;
  factory(url: string): EventSource;
  postMessage(url: string, message: MessageEvent): void;
}

export function mockEventSource(): Mess {
  const instances = new Map<
    string,
    { eventSource: EventSource; eventEmitter: EventEmitter }
  >();

  function MockEventSourceFactory(url: string): EventSource {
    assert(
      !instances.has(url),
      `EventSource instance already exists for ${url}`
    );

    const ee = new EventEmitter();
    const eventSource: EventSource = {
      addEventListener: jest.fn((type, callback) => {
        assert(typeof callback === 'function', 'callback must be a function');
        ee.on(type, callback);
      }) as unknown as EventSource['addEventListener'],

      removeEventListener: jest.fn((type, callback) => {
        assert(typeof callback === 'function', 'callback must be a function');
        ee.off(type, callback);
      }) as unknown as EventSource['removeEventListener'],

      close: jest.fn(() => {
        instances.delete(url);
        ee.removeAllListeners();
      }),

      set onmessage(callback: (event: MessageEvent) => void) {
        this.addEventListener('message', callback);
      },

      CLOSED: 2,
      CONNECTING: 0,
      OPEN: 1,

      set onerror(callback: (event: Event) => void) {
        this.addEventListener('error', callback);
      },

      set onopen(callback: (event: Event) => void) {
        this.addEventListener('open', callback);
      },

      url,
      withCredentials: false,
      readyState: 0,

      dispatchEvent(): boolean {
        return false;
      },
    };

    instances.set(url, {
      eventSource,
      eventEmitter: ee,
    });

    return eventSource;
  }

  return {
    install() {
      globalThis.EventSource =
        MockEventSourceFactory as unknown as typeof EventSource;
    },

    factory: MockEventSourceFactory,

    postMessage(url: string, message: MessageEvent) {
      const instance = instances.get(url);
      assert(instance, `No EventSource instance for ${url}`);
      instance.eventEmitter.emit('message', message);
    },
  };
}
