function ArrowPath({
  arrowWidth,
  arrowHeight,
}: {
  arrowWidth: number;
  arrowHeight: number;
}): JSX.Element {
  const arrowHeadHeight = arrowHeight / 2;
  const arrowShaftWidth = arrowWidth / 3;

  return (
    <path
      transform={`translate(${(arrowWidth - arrowWidth) / 2}, 0)`}
      d={[
        // Arrow head
        `M0 ${arrowHeadHeight}`,
        `L${arrowWidth / 2} 0`,
        `L${arrowWidth} ${arrowHeadHeight}`,
        `Z`,

        // Arrow shaft
        `M${arrowShaftWidth} ${arrowHeadHeight}`,
        `H${arrowWidth - arrowShaftWidth}`,
        `V${arrowHeight}`,
        `H${arrowShaftWidth}`,
        `Z`,
      ].join(' ')}
      fill="#A67ADF"
    />
  );
}

export function RemoveCommonAccessCardIcon(): JSX.Element {
  const width = 785;
  const height = 828;
  const arrowHeight = 120;
  const arrowWidth = 120;
  const cardMarginY = 50;
  const cardTop = arrowHeight + cardMarginY;
  const cardMarginX = 170;
  const cardWidth = width - cardMarginX * 2;
  const cardAndReaderHeight = height - cardTop;
  const cardStrokeWidth = 10;
  const cardVisibleHeight = 390;
  const readerStrokeWidth = cardStrokeWidth;
  const readerSlotStrokeWidth = 2;
  const readerSlotWidth = cardWidth + 50;
  const readerSlotHeight = 10;
  const readerSlotTop = cardVisibleHeight - readerSlotHeight;
  const readerSlotMidpoint = readerSlotTop + readerSlotHeight / 2;

  return (
    <svg viewBox={`0 0 ${width} ${height}`} transform="scale(0.75)">
      <g transform={`translate(0, ${cardTop})`}>
        {/* Reader */}
        <g>
          {/* Reader body */}
          <rect
            x={readerStrokeWidth / 2}
            y={readerStrokeWidth / 2}
            width={width - readerStrokeWidth}
            height={cardAndReaderHeight - readerStrokeWidth}
            fill="white"
            stroke="black"
            strokeWidth={readerStrokeWidth}
            mask="url(#readerBodyMask)"
          />
          <mask id="readerBodyMask">
            <rect
              x="0"
              y={cardVisibleHeight}
              width="100%"
              height="100%"
              fill="white"
            />
          </mask>

          {/* Reader top */}
          <ellipse
            cx={width / 2}
            cy={cardVisibleHeight}
            rx={(width - readerStrokeWidth) / 2}
            ry="50"
            fill="white"
            stroke="black"
            strokeWidth={readerStrokeWidth}
          />
          <ellipse
            cx={width / 2}
            cy={cardVisibleHeight + 15}
            rx={(width - readerStrokeWidth) / 2}
            ry="65"
            fill="none"
            stroke="black"
            strokeWidth={readerStrokeWidth}
            mask="url(#readerTopOuterMask)"
          />
          <mask id="readerTopOuterMask">
            <rect
              x="0"
              y={readerSlotMidpoint + 20}
              width="100%"
              height="100%"
              fill="white"
            />
          </mask>
        </g>

        {/* Card */}
        <g transform={`translate(${cardMarginX}, 0)`}>
          {/* Card border */}
          <rect
            x="0"
            y={cardStrokeWidth / 2}
            width={cardWidth}
            height={cardAndReaderHeight - cardStrokeWidth}
            fill="white"
            stroke="black"
            strokeWidth={cardStrokeWidth}
            rx="20"
            ry="20"
            mask="url(#cardMask)"
          />
          <mask id="cardMask">
            <rect
              x={-cardStrokeWidth / 2}
              y="0"
              width="100%"
              height={cardVisibleHeight}
              fill="white"
            />
          </mask>

          {/* Person head */}
          <circle cx="290" cy="210" r="60" fill="black" />

          {/* Person body */}
          <ellipse
            cx="90"
            cy="210"
            rx="110"
            ry="142"
            fill="black"
            mask="url(#personBodyMask)"
          />
          <mask id="personBodyMask">
            <rect x="90" y="0" width="110" height={height} fill="white" />
          </mask>
        </g>

        {/* Reader slot – needs to be drawn on top of the card */}
        <rect
          x={(width - readerSlotWidth) / 2}
          y={readerSlotTop}
          width={readerSlotWidth}
          height={readerSlotHeight}
          fill="#cccccc"
          stroke="black"
          strokeWidth={readerSlotStrokeWidth}
          rx="10"
          mask="url(#readerSlotMask)"
          z=""
        />
        <mask id="readerSlotMask">
          <rect x="0" y="0" width="100%" height="100%" fill="white" />
          <rect
            x={cardMarginX - cardStrokeWidth / 2}
            y={cardVisibleHeight - readerSlotHeight - readerSlotStrokeWidth}
            width={cardWidth + cardStrokeWidth}
            height={readerSlotHeight + readerSlotStrokeWidth / 2}
            fill="black"
          />
        </mask>
      </g>

      <g transform={`translate(${(width - arrowWidth) / 2}, 0)`}>
        <ArrowPath arrowWidth={arrowWidth} arrowHeight={arrowHeight} />
      </g>
    </svg>
  );
}
