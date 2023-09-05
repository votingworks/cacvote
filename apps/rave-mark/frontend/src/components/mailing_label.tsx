/* eslint-disable vx/gts-identifiers */

import React, { useLayoutEffect, useRef } from 'react';
import styled from 'styled-components';

const Container = styled.div`
  @page {
    size: 4in 6in;
    margin: 0;
  }

  @media screen {
    display: none;
  }

  font-family: Arial, sans-serif;
  width: 4in;
  height: 6in;
  padding: 0.26in 0.12in;
  box-sizing: border-box;
`;

const VStack = styled.div<{ separator?: boolean }>`
  display: flex;
  flex-direction: column;

  > * {
    border-bottom: ${({ separator }) =>
      separator ? '3px solid black' : 'none'};
  }

  > *:last-child {
    border-bottom: none;
  }
`;

const HStack = styled.div<{ separator?: boolean }>`
  display: flex;
  flex-direction: row;

  > * {
    border-right: ${({ separator }) =>
      separator ? '3px solid black' : 'none'};
  }

  > *:last-child {
    border-right: none;
  }
`;

const InsetContainer = styled.div`
  width: 100%;
  height: 100%;
  border: 4px solid black;
  box-sizing: border-box;
`;

const LargeP = styled.div`
  font-size: 1in;
  font-weight: bold;
  text-align: center;
  color: white;
  padding: 0 0.05in;
  -webkit-text-stroke: 3px black;
  ::after {
    content: 'P';
  }
`;

const MainBody = styled.div`
  height: 2.5in;
`;

const PriorityMailLabel = styled.div`
  font-size: 0.22in;
  padding: 0.05in 0.2in;
  font-weight: bold;
  text-transform: uppercase;
  text-align: center;
  ::after {
    content: 'USPS Priority Mail®';
  }
`;

const TrackingNumberHeading = styled.div`
  font-size: 0.12in;
  font-weight: bold;
  text-align: center;
  text-transform: uppercase;
  padding: 0.031in 0.2in;
  ::after {
    content: 'USPS Tracking #';
  }
`;

const TrackingNumberBarcodeInner = styled.canvas`
  width: 100%;
  height: 1in;
`;

function TrackingNumberBarcode(): JSX.Element {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useLayoutEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) {
      return;
    }

    const ctx = canvas.getContext('2d');
    if (!ctx) {
      return;
    }

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    ctx.fillStyle = 'black';
    const { width, height } = canvas;
    const padding = Math.round(width / 20);
    let bias = 0;

    for (let i = padding; i < width - padding * 2; i += 1) {
      if (Math.random() < 0.5 - bias) {
        ctx.fillRect(i, 0, 1, height);
        bias += 0.1;
      } else {
        bias -= 0.1;
      }
    }
  }, []);

  return <TrackingNumberBarcodeInner ref={canvasRef} />;
}

interface AddressProps {
  address: MailingAddress;
  scale?: number;
}

function Address({ address, scale = 1 }: AddressProps): JSX.Element {
  const Wrapper = styled.div`
    font-size: ${() => 0.13 * scale}in;
    padding: 0.05in 0.1in;
    font-weight: bold;
  `;

  return (
    <Wrapper>
      {address.addresseeLine1}
      <br />
      {address.addresseeLine2 && (
        <React.Fragment>
          {address.addresseeLine2}
          <br />
        </React.Fragment>
      )}
      {address.addressLine1}
      <br />
      {address.addressLine2 && (
        <React.Fragment>
          {address.addressLine2}
          <br />
        </React.Fragment>
      )}
      {address.city}, {address.state} {address.postalCode}
    </Wrapper>
  );
}

const TrackingNumberLabel = styled.div`
  font-size: 0.12in;
  font-weight: bold;
  text-align: center;
  text-transform: uppercase;
  padding: 0.031in 0.2in;
`;

function TrackingNumber(): JSX.Element {
  return (
    <VStack>
      <TrackingNumberHeading />
      <TrackingNumberBarcode />
      <TrackingNumberLabel>9400 1000 0000 0000 0000 00</TrackingNumberLabel>
    </VStack>
  );
}

interface MailingAddress {
  addresseeLine1: string;
  addresseeLine2?: string;
  addressLine1: string;
  addressLine2?: string;
  city: string;
  state: string;
  postalCode: string;
}

interface ShipFromAddressProps {
  address: MailingAddress;
}

function ShippingAddress({ address }: ShipFromAddressProps): JSX.Element {
  const Wrapper = styled.div`
    margin-top: 0in;
    margin-bottom: 0.3in;
  `;

  return (
    <Wrapper>
      <Address address={address} />
    </Wrapper>
  );
}

function ShipToAddress(): JSX.Element {
  const ShipToLabel = styled.div`
    font-size: 0.12in;
    padding: 0.05in 0.2in;
    font-weight: bold;
    text-transform: uppercase;
    width: 5em;
    ::after {
      content: 'Ship to:';
    }
  `;

  return (
    <HStack>
      <ShipToLabel />
      <Address
        address={{
          addresseeLine1: 'Ballot Receiving Center',
          addressLine1: '1234 Main St',
          city: 'Anytown',
          state: 'CA',
          postalCode: '12345',
        }}
        scale={1.4}
      />
    </HStack>
  );
}

function QRCode(): JSX.Element {
  const Wrapper = styled.div`
    display: flex;
    justify-content: flex-end;
    align-items: center;
    flex-grow: 1;
  `;

  return (
    <Wrapper>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        version="1.1"
        width={80}
        height={70}
      >
        <path
          d="m 16,16 0,16 0,16 0,16 0,16 0,16 0,16 0,16 16,0 16,0 16,0 16,0 16,0 16,0 16,0 0,-16 0,-16 0,-16 0,-16 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 -16,0 -16,0 -16,0 -16,0 z m 128,0 0,16 0,16 16,0 0,-16 16,0 0,-16 -16,0 -16,0 z m 32,16 0,16 16,0 0,-16 -16,0 z m 16,16 0,16 16,0 16,0 0,-16 0,-16 0,-16 -16,0 0,16 0,16 -16,0 z m 0,16 -16,0 -16,0 -16,0 0,16 16,0 16,0 0,16 -16,0 0,16 16,0 0,16 16,0 0,-16 16,0 0,16 -16,0 0,16 -16,0 0,16 16,0 16,0 0,16 16,0 0,-16 0,-16 0,-16 0,-16 0,-16 -16,0 0,-16 -16,0 0,-16 z m 16,112 -16,0 0,16 -16,0 0,16 0,16 16,0 0,16 0,16 -16,0 -16,0 0,-16 16,0 0,-16 -16,0 0,-16 0,-16 -16,0 0,-16 16,0 0,16 16,0 0,-16 0,-16 -16,0 -16,0 0,-16 -16,0 -16,0 -16,0 0,16 -16,0 0,16 -16,0 0,-16 16,0 0,-16 -16,0 -16,0 0,16 -16,0 0,16 0,16 0,16 16,0 0,-16 16,0 16,0 16,0 0,-16 16,0 0,-16 16,0 0,16 -16,0 0,16 16,0 0,16 -16,0 0,16 16,0 16,0 0,16 0,16 0,16 16,0 0,16 16,0 16,0 16,0 0,16 -16,0 -16,0 -16,0 0,-16 -16,0 0,16 0,16 16,0 0,16 -16,0 0,16 16,0 16,0 0,-16 16,0 0,16 16,0 16,0 16,0 16,0 0,-16 16,0 0,16 16,0 16,0 16,0 0,-16 -16,0 -16,0 0,-16 -16,0 0,-16 -16,0 0,16 -16,0 -16,0 0,16 -16,0 0,-16 16,0 0,-16 0,-16 0,-16 16,0 0,-16 -16,0 -16,0 0,-16 16,0 0,-16 0,-16 0,-16 -16,0 0,-16 z m 48,128 0,-16 -16,0 0,16 16,0 z m 32,16 16,0 16,0 0,-16 -16,0 -16,0 0,16 z m 32,-16 16,0 0,-16 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 0,-16 -16,0 0,16 -16,0 0,16 0,16 16,0 0,-16 16,0 0,16 0,16 16,0 16,0 0,16 z m -48,-80 0,-16 -16,0 -16,0 0,16 16,0 16,0 z m 16,0 16,0 0,-16 0,-16 0,-16 16,0 0,16 16,0 0,16 16,0 0,-16 0,-16 -16,0 0,-16 16,0 0,-16 -16,0 -16,0 0,16 -16,0 0,-16 -16,0 0,16 -16,0 0,16 16,0 0,16 0,16 0,16 z m -16,-48 -16,0 0,16 16,0 0,-16 z m 64,32 -16,0 0,16 16,0 0,-16 z m -224,0 0,-16 -16,0 0,16 16,0 z m -16,0 -16,0 -16,0 -16,0 0,16 16,0 16,0 16,0 0,-16 z m -64,0 -16,0 0,16 16,0 0,-16 z m 0,-48 0,-16 -16,0 0,16 16,0 z m 112,-16 16,0 0,-16 0,-16 -16,0 0,16 0,16 z m 96,-128 0,16 0,16 0,16 0,16 0,16 0,16 0,16 16,0 16,0 16,0 16,0 16,0 16,0 16,0 0,-16 0,-16 0,-16 0,-16 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 -16,0 -16,0 -16,0 -16,0 z m -208,16 16,0 16,0 16,0 16,0 16,0 0,16 0,16 0,16 0,16 0,16 -16,0 -16,0 -16,0 -16,0 -16,0 0,-16 0,-16 0,-16 0,-16 0,-16 z m 224,0 16,0 16,0 16,0 16,0 16,0 0,16 0,16 0,16 0,16 0,16 -16,0 -16,0 -16,0 -16,0 -16,0 0,-16 0,-16 0,-16 0,-16 0,-16 z m -208,16 0,16 0,16 0,16 16,0 16,0 16,0 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 z m 224,0 0,16 0,16 0,16 16,0 16,0 16,0 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 z m -32,96 0,16 16,0 0,-16 -16,0 z m -224,96 0,16 0,16 0,16 0,16 0,16 0,16 0,16 16,0 16,0 16,0 16,0 16,0 16,0 16,0 0,-16 0,-16 0,-16 0,-16 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 -16,0 -16,0 -16,0 -16,0 z m 16,16 16,0 16,0 16,0 16,0 16,0 0,16 0,16 0,16 0,16 0,16 -16,0 -16,0 -16,0 -16,0 -16,0 0,-16 0,-16 0,-16 0,-16 0,-16 z m 16,16 0,16 0,16 0,16 16,0 16,0 16,0 0,-16 0,-16 0,-16 -16,0 -16,0 -16,0 z m 288,48 0,16 16,0 0,-16 -16,0 z"
          transform="scale(0.20)"
          style={{ fill: '#000000', stroke: 'none' }}
        />
      </svg>
    </Wrapper>
  );
}

const OfficialElectionMailLabel = styled.div`
  font-size: 0.275in;
  padding: 0.05in 0.2in;
  text-transform: uppercase;
  align-self: center;
  ::after {
    content: 'Official Election Mail';
  }
`;

export function MailingLabel(): JSX.Element {
  return (
    <Container>
      <InsetContainer>
        <VStack separator>
          <HStack separator>
            <LargeP />
            <HStack>
              <OfficialElectionMailLabel />
              <QRCode />
            </HStack>
          </HStack>
          <PriorityMailLabel />
          <MainBody>
            <VStack>
              <ShippingAddress
                address={{
                  addresseeLine1: 'Jane Doe',
                  addresseeLine2: 'Example Military Base',
                  addressLine1: '1234 Main St',
                  city: 'Anytown',
                  state: 'CA',
                  postalCode: '12345',
                }}
              />
              <ShipToAddress />
            </VStack>
          </MainBody>
          <TrackingNumber />
        </VStack>
      </InsetContainer>
    </Container>
  );
}
