import { Icon, IconName } from "@/components/Icon";
import React from "react";
import {
  getUnitIcon,
  renderUndefinedValue,
  renderUnitSymbol,
  Unit,
} from "./units";
import { Label } from "./Label";
import { TimeSeries } from "@/lib/timeseries";
import { MiniGraph } from "@/helpers/MiniGraph";

type Props = {
  label: string;
  unit?: Unit;
  timeseries: TimeSeries;
  icon?: IconName;
  renderValue?: (value: number) => string;
};

type AllowedT = number | boolean;


function _TimeSeriesValue({
  unit,
  timeseries,
  icon,
  label,
  renderValue,
}: Props) {
  const value = timeseries.current?.value;

  const containerRef = React.useRef<HTMLDivElement>(null);
  const leftRef = React.useRef<HTMLDivElement>(null);
  const [width, setWidth] = React.useState(0);

  React.useEffect(() => {
    if (!containerRef.current || !leftRef.current) return;

    const container = containerRef.current;
    const left = leftRef.current;

    const calculateWidth = () => {
      const containerWidth = container.getBoundingClientRect().width;
      const leftWidth = left.getBoundingClientRect().width;
      const padding = 16;
      const remaining = Math.max(containerWidth - leftWidth - padding, 0);
      setWidth(remaining);
    };

    calculateWidth();

    // Listen for resizes on the container
    const resizeObserver = new ResizeObserver(() => {
      calculateWidth();
    });

    resizeObserver.observe(container);
    resizeObserver.observe(left);

    return () => {
      resizeObserver.disconnect();
    };
  }, []);

  return (
    <div
      ref={containerRef}
      className="bg-red flex flex-row items-center gap-4 w-full"
    >
      <div ref={leftRef} className="h-16 flex-shrink-0">
        <Label label={label}>
          <div className="flex flex-row items-center gap-4">
            <Icon
              name={icon ?? (unit ? getUnitIcon(unit) : undefined)}
              className="size-7"
            />
            <div className="flex flex-row items-center gap-2">
              <span className="font-mono text-4xl font-bold">
                {renderUndefinedValue(value, unit, renderValue)}
              </span>
              <span>{renderUnitSymbol(unit)}</span>
            </div>
          </div>
        </Label>
      </div>

      <div className="h-16 flex-grow min-w-0">
        <MiniGraph newData={timeseries} width={width} renderValue={renderValue} />
      </div>
    </div>
  );
}


export function TimeSeriesValueNumeric(props: Props) {
  return <_TimeSeriesValue {...props} />;
}
