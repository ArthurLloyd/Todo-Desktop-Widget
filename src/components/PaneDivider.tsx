import type { KeyboardEvent, PointerEvent } from "react";

type Props = {
  orientation: "vertical" | "horizontal";
  label: string;
  valueNow: number;
  onPointerStart: (event: PointerEvent<HTMLDivElement>) => void;
  onPointerMove: (event: PointerEvent<HTMLDivElement>) => void;
  onPointerEnd: (event: PointerEvent<HTMLDivElement>) => void;
  onKeyResize: (event: KeyboardEvent<HTMLDivElement>) => void;
};

export function PaneDivider({
  orientation,
  label,
  valueNow,
  onPointerStart,
  onPointerMove,
  onPointerEnd,
  onKeyResize,
}: Props) {
  return (
    <div
      className="pane-divider"
      data-orientation={orientation}
      role="separator"
      aria-label={label}
      aria-orientation={orientation}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(valueNow)}
      tabIndex={0}
      onPointerDown={(event) => {
        event.currentTarget.setPointerCapture(event.pointerId);
        onPointerStart(event);
      }}
      onPointerMove={onPointerMove}
      onPointerUp={(event) => {
        onPointerEnd(event);
        if (event.currentTarget.hasPointerCapture(event.pointerId)) {
          event.currentTarget.releasePointerCapture(event.pointerId);
        }
      }}
      onPointerCancel={onPointerEnd}
      onLostPointerCapture={onPointerEnd}
      onKeyDown={onKeyResize}
    >
      <span aria-hidden="true" />
    </div>
  );
}
