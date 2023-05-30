import {
  faCamera,
  faCaretDown,
  faCaretUp,
  faCircleDown,
} from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import {
  QueryClient,
  QueryClientProvider,
  useMutation,
  useQuery,
  useQueryClient,
} from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { assert } from '@votingworks/basics';
import type { Api, DevDockUserRole } from '@votingworks/dev-dock-backend';
import * as grout from '@votingworks/grout';
import { Id } from '@votingworks/types';
import {
  BooleanEnvironmentVariableName,
  isFeatureFlagEnabled,
} from '@votingworks/utils';
import React, { RefObject, useEffect, useRef, useState } from 'react';
import styled from 'styled-components';
import { Colors } from './colors';
import { UsbDriveIcon } from './usb_drive_icon';

export type ApiClient = grout.Client<Api>;

export const ApiClientContext = React.createContext<ApiClient | undefined>(
  undefined
);

export function useApiClient(): ApiClient {
  const apiClient = React.useContext(ApiClientContext);
  if (!apiClient) {
    throw new Error('ApiClientContext.Provider not found');
  }
  return apiClient;
}

const Row = styled.div`
  display: flex;
  flex-direction: row;
  gap: 15px;
`;

const Column = styled.div`
  display: flex;
  flex-direction: column;
  gap: 15px;
`;

const SmartCardButton = styled.button<{ isInserted: boolean }>`
  background-color: white;
  border: ${(props) =>
    props.isInserted
      ? `4px solid ${Colors.ACTIVE}`
      : `1px solid ${Colors.BORDER}`};
  color: ${(props) => (props.isInserted ? Colors.ACTIVE : Colors.TEXT)};
  border-radius: 8px;
  width: 115px;
  height: 175px;
  display: flex;
  flex-direction: column;
  align-items: center;
  p {
    font-weight: bold;
    font-size: 0.85em;
    margin-bottom: 40px;
  }
  &:disabled {
    color: ${Colors.DISABLED};
    border-color: ${Colors.DISABLED};
  }
`;

function SmartCardControl({
  label,
  isInserted,
  onClick,
  disabled,
}: {
  label: string;
  isInserted: boolean;
  onClick(): void;
  disabled: boolean;
}): JSX.Element {
  return (
    <SmartCardButton
      onClick={onClick}
      isInserted={isInserted}
      disabled={disabled}
    >
      <p>{label}</p>
      {isInserted && <FontAwesomeIcon icon={faCircleDown} size="lg" />}
    </SmartCardButton>
  );
}

const SmartCardMocksDisabledMessage = styled.div`
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  justify-content: center;
  > p {
    padding: 15px;
    background: #cccccc;
    text-align: center;
  }
`;

function SmartCardMockControls() {
  const queryClient = useQueryClient();
  const apiClient = useApiClient();
  const getCardStatusQuery = useQuery(
    ['getCardStatus'],
    async () => (await apiClient.getCardStatus()) ?? null
  );
  const insertCardMutation = useMutation(apiClient.insertCard, {
    onSuccess: async () =>
      await queryClient.invalidateQueries(['getCardStatus']),
  });
  const removeCardMutation = useMutation(apiClient.removeCard, {
    onSuccess: async () =>
      await queryClient.invalidateQueries(['getCardStatus']),
  });
  const voters: Array<{ commonAccessCardId: string }> = [
    { commonAccessCardId: 'User 1' },
    { commonAccessCardId: 'User 2' },
    { commonAccessCardId: 'User 3' },
    { commonAccessCardId: 'User 4' },
    { commonAccessCardId: 'User 5' },
  ];

  const cardStatus = getCardStatusQuery.data;
  const insertedCardUser =
    cardStatus?.status === 'ready' ? cardStatus.cardDetails?.user : undefined;
  const insertedCardRole = insertedCardUser?.role;

  function onCardClick(role: DevDockUserRole, commonAccessCardId?: Id) {
    if (insertedCardRole === role) {
      removeCardMutation.mutate();
    } else {
      insertCardMutation.mutate({ role, commonAccessCardId });
    }
  }

  const areSmartCardMocksEnabled = isFeatureFlagEnabled(
    BooleanEnvironmentVariableName.USE_MOCK_CARDS
  );

  return (
    <Row style={{ position: 'relative' }}>
      {!areSmartCardMocksEnabled && (
        <SmartCardMocksDisabledMessage>
          <p>
            Smart card mocks disabled
            <br />
            <code>USE_MOCK_CARDS=FALSE</code>
          </p>
        </SmartCardMocksDisabledMessage>
      )}
      {voters.map(({ commonAccessCardId }) => (
        <SmartCardControl
          key={commonAccessCardId}
          isInserted={
            insertedCardUser?.role === 'rave_voter' &&
            insertedCardUser.commonAccessCardId === commonAccessCardId
          }
          label={commonAccessCardId}
          onClick={() => onCardClick('rave_voter', commonAccessCardId)}
          disabled={
            !areSmartCardMocksEnabled ||
            !getCardStatusQuery.isSuccess ||
            (insertedCardUser !== undefined &&
              (insertedCardUser?.role !== 'rave_voter' ||
                insertedCardUser?.commonAccessCardId !== commonAccessCardId))
          }
        />
      ))}
    </Row>
  );
}

const UsbDriveControl = styled.button<{ isInserted: boolean }>`
  background-color: white;
  width: 80px;
  height: 120px;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 5px;
  border: ${(props) =>
    props.isInserted
      ? `4px solid ${Colors.ACTIVE}`
      : `1px solid ${Colors.BORDER}`};
`;

const UsbDriveClearButton = styled.button`
  background-color: white;
  border: 1px solid ${Colors.BORDER};
  border-radius: 4px;
  width: 100%;
  height: 40px;
  color: ${Colors.TEXT};
  &:active {
    color: ${Colors.ACTIVE};
    border-color: ${Colors.ACTIVE};
  }
`;

function UsbDriveMockControls() {
  const queryClient = useQueryClient();
  const apiClient = useApiClient();
  const getUsbDriveStatusQuery = useQuery(['getUsbDriveStatus'], () =>
    apiClient.getUsbDriveStatus()
  );
  const insertUsbDriveMutation = useMutation(apiClient.insertUsbDrive, {
    onSuccess: async () =>
      await queryClient.invalidateQueries(['getUsbDriveStatus']),
  });
  const removeUsbDriveMutation = useMutation(apiClient.removeUsbDrive, {
    onSuccess: async () =>
      await queryClient.invalidateQueries(['getUsbDriveStatus']),
  });
  const clearUsbDriveMutation = useMutation(apiClient.clearUsbDrive, {
    onSuccess: async () =>
      await queryClient.invalidateQueries(['getUsbDriveStatus']),
  });

  const status = getUsbDriveStatusQuery.data ?? undefined;

  function onUsbDriveClick() {
    if (status === 'inserted') {
      removeUsbDriveMutation.mutate();
    } else {
      insertUsbDriveMutation.mutate();
    }
  }

  function onClearUsbDriveClick() {
    clearUsbDriveMutation.mutate();
  }

  const isInserted = status === 'inserted';
  return (
    <Column>
      <UsbDriveControl
        onClick={onUsbDriveClick}
        isInserted={isInserted}
        aria-label="USB Drive"
      >
        {status && <UsbDriveIcon isInserted={isInserted} />}
      </UsbDriveControl>
      <UsbDriveClearButton onClick={onClearUsbDriveClick}>
        Clear
      </UsbDriveClearButton>
    </Column>
  );
}

const ScreenshotButton = styled.button`
  background-color: white;
  width: 80px;
  height: 80px;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border: 1px solid ${Colors.BORDER};
  color: ${Colors.TEXT};

  &:active {
    color: ${Colors.ACTIVE};
    border-color: ${Colors.ACTIVE};
  }
  &:disabled {
    color: ${Colors.DISABLED};
    border-color: ${Colors.DISABLED};
  }
`;

function ScreenshotControls({
  containerRef,
}: {
  containerRef: RefObject<HTMLDivElement>;
}) {
  async function captureScreenshot() {
    // Use a ref to the dock container to momentarily hide it during the
    // screenshot.
    assert(containerRef.current);
    // eslint-disable-next-line no-param-reassign
    containerRef.current.style.visibility = 'hidden';

    assert(window.kiosk);
    const screenshotData = await window.kiosk.captureScreenshot();

    // eslint-disable-next-line no-param-reassign
    containerRef.current.style.visibility = 'visible';

    // "VotingWorks VxAdmin" -> "VxAdmin"
    const appName = document.title.replace('VotingWorks', '').trim();
    const fileName = `Screenshot-${appName}-${new Date().toISOString()}.png`;
    const saveFile = await window.kiosk.saveAs({
      defaultPath: fileName,
    });
    await saveFile?.write(screenshotData);
  }

  return (
    <ScreenshotButton
      onClick={captureScreenshot}
      disabled={!window.kiosk}
      aria-label="Capture Screenshot"
    >
      <FontAwesomeIcon icon={faCamera} size="2x" />
    </ScreenshotButton>
  );
}

const Container = styled.div`
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  z-index: 1000; /* Above react-modal z-index of 999 */
  pointer-events: none;
  /* Draw a unified shadow around the content and handle */
  filter: drop-shadow(0 4px 10px rgba(0, 0, 0, 0.35))
    drop-shadow(0 0 2px #291649);

  @media print {
    display: none; /* Do not print the dock */
  }
  *:focus {
    outline: none;
  }

  /* Slide the dock up when closed */
  &.closed {
    /* Slide up enough to hide the shadow */
    top: -300px;
    transition: top 0.15s ease-out;
    /* Move the handle down a bit to compensate for sliding up extra to hide the
     * shadow */
    #handle {
      top: 20px;
    }
  }
  /* Slide the dock down when open */
  transition: top 0.15s ease-out;
`;

const Content = styled.div`
  font-size: 24px !important;
  background-color: ${Colors.BACKGROUND};
  padding: 15px 15px 20px 15px;
  display: flex;
  flex-direction: column;
  gap: 15px;
  pointer-events: auto;
  border-radius: 0px 0px 10px 10px;
`;

const Handle = styled.button`
  background-color: ${Colors.BACKGROUND};
  height: 60px;
  width: 100px;
  border-width: 0;
  pointer-events: auto;
  border-radius: 0px 0px 10px 10px;
  position: relative;
  /* Overlap with content so that filter shadow is not visible */
  top: -2px;
`;

function createQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        networkMode: 'always',
        staleTime: Infinity,
        onError: (error) => {
          // eslint-disable-next-line no-console
          console.error('Dev Dock error:', error);
        },
      },
      mutations: {
        networkMode: 'always',
        onError: (error) => {
          // eslint-disable-next-line no-console
          console.error('Dev Dock error:', error);
        },
      },
    },
  });
}

function DevDock() {
  const [isOpen, setIsOpen] = useState(true);
  const containerRef = useRef<HTMLDivElement>(null);

  function onKeyDown(event: KeyboardEvent): void {
    if (event.key === 'd' && event.metaKey) {
      setIsOpen((previousIsOpen) => !previousIsOpen);
      event.preventDefault();
    }
    if (isOpen) {
      if (event.key === 'Escape') setIsOpen(false);
    }
  }

  useEffect(() => {
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, []);

  return (
    <Container ref={containerRef} className={isOpen ? '' : 'closed'}>
      <Content>
        <Row>
          <Column>
            <Row>
              <SmartCardMockControls />
            </Row>
          </Column>
          <Column>
            <UsbDriveMockControls />
          </Column>
          <Column>
            <ScreenshotControls containerRef={containerRef} />
          </Column>
        </Row>
      </Content>
      <Handle id="handle" onClick={() => setIsOpen(!isOpen)}>
        <FontAwesomeIcon icon={isOpen ? faCaretUp : faCaretDown} size="lg" />
      </Handle>
    </Container>
  );
}

/**
 * Dev dock component. Render at the top level of an app.
 *
 * The dock will only be rendered if the ENABLE_DEV_DOCK feature flag is turned
 * on.
 */
function DevDockWrapper({
  apiClient = grout.createClient<Api>({ baseUrl: '/dock' }),
}: {
  apiClient?: ApiClient;
}): JSX.Element | null {
  // We use a wrapper component to make sure that not only is the dock not
  // inserted into the DOM, but its keyboard listeners are not registered
  // either.
  return isFeatureFlagEnabled(
    BooleanEnvironmentVariableName.ENABLE_DEV_DOCK
  ) ? (
    <QueryClientProvider client={createQueryClient()}>
      <ApiClientContext.Provider value={apiClient}>
        <DevDock />
        {false && <ReactQueryDevtools initialIsOpen={false} />}
      </ApiClientContext.Provider>
    </QueryClientProvider>
  ) : null;
}

export { DevDockWrapper as DevDock };
