import { dirSync } from 'tmp';
import { createWorkspace } from './workspace';
import { Store } from './store';

test('workspace.reset rests the store', () => {
  const workspace = createWorkspace(dirSync().name, Store);
  const fn = jest.fn();
  workspace.store.reset = fn;
  workspace.reset();
  expect(fn).toHaveBeenCalledTimes(1);
});
