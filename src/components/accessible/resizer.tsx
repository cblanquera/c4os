import type { HTMLAttributes, KeyboardEvent, PointerEvent } from "react";
import { useRef, useState } from "react";

export interface ResizerProps extends Omit<
  HTMLAttributes<HTMLDivElement>,
  | "aria-label"
  | "aria-orientation"
  | "aria-valuemax"
  | "aria-valuemin"
  | "aria-valuenow"
  | "onChange"
  | "onKeyDown"
  | "onPointerCancel"
  | "onPointerDown"
  | "onPointerMove"
  | "onPointerUp"
  | "role"
> {
  readonly direction?: 1 | -1;
  readonly formatValue?: (value: number) => string;
  readonly label: string;
  readonly max: number;
  readonly min: number;
  readonly onValueChange: (value: number) => void;
  readonly orientation: "horizontal" | "vertical";
  readonly step?: number;
  readonly value: number;
}

type PointerOrigin = {
  readonly coordinate: number;
  readonly pointerId: number;
  readonly value: number;
};

/** Keeps a proposed resize value inside the component's accepted range. */
function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}

/** Renders a pointer and keyboard separator with an assistive value contract. */
export function Resizer({
  className,
  direction = 1,
  formatValue = (currentValue) => `${currentValue} pixels`,
  label,
  max,
  min,
  onValueChange,
  orientation,
  step = 8,
  value,
  ...props
}: ResizerProps) {
  const pointerOrigin = useRef<PointerOrigin | null>(null);
  const [isResizing, setIsResizing] = useState(false);

  /** Resolves the relevant pointer coordinate from the separator orientation. */
  const pointerCoordinate = (event: PointerEvent<HTMLDivElement>) =>
    orientation === "vertical" ? event.clientX : event.clientY;

  /** Starts one captured pointer gesture without attaching document listeners. */
  const handlePointerDown = (event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) {
      return;
    }

    pointerOrigin.current = {
      coordinate: pointerCoordinate(event),
      pointerId: event.pointerId,
      value,
    };
    event.currentTarget.setPointerCapture?.(event.pointerId);
    event.currentTarget.focus();
    setIsResizing(true);
  };

  /** Applies the captured pointer delta to the current controlled value. */
  const handlePointerMove = (event: PointerEvent<HTMLDivElement>) => {
    const origin = pointerOrigin.current;
    if (!origin || origin.pointerId !== event.pointerId) {
      return;
    }

    const delta = (pointerCoordinate(event) - origin.coordinate) * direction;
    onValueChange(clamp(origin.value + delta, min, max));
  };

  /** Ends the active pointer gesture while preserving the committed value. */
  const endPointerGesture = (event: PointerEvent<HTMLDivElement>) => {
    const origin = pointerOrigin.current;
    if (!origin || origin.pointerId !== event.pointerId) {
      return;
    }

    event.currentTarget.releasePointerCapture?.(event.pointerId);
    pointerOrigin.current = null;
    setIsResizing(false);
  };

  /** Maps directional, range, and larger-step keys to a bounded value. */
  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const decreaseKey = orientation === "vertical" ? "ArrowLeft" : "ArrowUp";
    const increaseKey = orientation === "vertical" ? "ArrowRight" : "ArrowDown";
    let nextValue: number | null = null;

    if (event.key === decreaseKey) {
      nextValue = value - step * direction;
    } else if (event.key === increaseKey) {
      nextValue = value + step * direction;
    } else if (event.key === "PageDown") {
      nextValue = value - step * 5 * direction;
    } else if (event.key === "PageUp") {
      nextValue = value + step * 5 * direction;
    } else if (event.key === "Home") {
      nextValue = min;
    } else if (event.key === "End") {
      nextValue = max;
    }

    if (nextValue === null) {
      return;
    }

    event.preventDefault();
    onValueChange(clamp(nextValue, min, max));
  };

  return (
    <div
      {...props}
      aria-label={label}
      aria-orientation={orientation}
      aria-valuemax={max}
      aria-valuemin={min}
      aria-valuenow={value}
      aria-valuetext={formatValue(value)}
      className={["c4-resizer", `c4-resizer--${orientation}`, className]
        .filter(Boolean)
        .join(" ")}
      data-resizing={isResizing || undefined}
      onKeyDown={handleKeyDown}
      onPointerCancel={endPointerGesture}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={endPointerGesture}
      role="separator"
      tabIndex={0}
    >
      <span aria-hidden="true" className="c4-resizer__handle" />
    </div>
  );
}
