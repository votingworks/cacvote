import { useEffect } from 'react';

export interface UseModalKeybindingsProps {
  onEnter: () => void;
  onEscape: () => void;
}

/**
 * Handles common keybindings for modal dialogs.
 */
export function useModalKeybindings({
  onEnter,
  onEscape,
}: UseModalKeybindingsProps): void {
  useEffect(() => {
    function onKeyUp(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onEscape();
        return;
      }

      const focusedElement = document.activeElement;
      const focusedElementIsInput =
        focusedElement instanceof HTMLInputElement ||
        focusedElement instanceof HTMLTextAreaElement ||
        focusedElement instanceof HTMLSelectElement;

      if (!focusedElementIsInput) {
        if (event.key === 'Enter') {
          onEnter();
        }
      }
    }

    window.addEventListener('keyup', onKeyUp);

    return () => {
      window.removeEventListener('keyup', onKeyUp);
    };
  }, [onEnter, onEscape]);
}
