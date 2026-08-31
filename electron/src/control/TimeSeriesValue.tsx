import { Icon, IconName } from "@/components/Icon";
import React from "react";
import {
  getUnitIcon,
  renderValueToReactNode,
  renderUnitSymbol,
  Unit,
} from "./units";
import { Label } from "./Label";
import { TimeSeries } from "@/lib/timeseries";
import { MiniGraph } from "@/components/graph/MiniGraph";
import {
  useContainerDimensions,
  useMaxContainerMaxDimension,
} from "@/lib/useContainerWidth";

type Props = {
  label: string;
  unit?: Unit;
  timeseries: TimeSeries;
  icon?: IconName;
  renderValue?: (value: number) => string;
};

function _TimeSeriesValue({
  unit,
  timeseries,
  icon,
  label,
  renderValue,
}: Props) {
  const value = timeseries.current?.value;

  const leftRef = React.useRef<HTMLDivElement>(null);
  const minigraphContainerRef = React.useRef<HTMLDivElement>(null);

  // observe width of minigraph container
  const { width } = useContainerDimensions(minigraphContainerRef);

  // observe max width of left side
  const { maxWidth: leftMaxWidth } = useMaxContainerMaxDimension(leftRef);

  return (
    <div className="flex w-full flex-row items-center gap-4">
      <div
        ref={leftRef}
        className="h-16"
        style={{ minWidth: `${leftMaxWidth}px` }}
      >
        <Label label={label}>
          <div className="flex flex-row items-center gap-4">
            <Icon
              name={icon ?? (unit ? getUnitIcon(unit) : undefined)}
              className="size-7"
            />
            <div className="flex flex-row items-center gap-2">
              <span className="font-mono text-4xl font-bold">
                {renderValueToReactNode(value, unit, renderValue)}
              </span>
              <span>{renderUnitSymbol(unit)}</span>
            </div>
          </div>
        </Label>
      </div>

      <div ref={minigraphContainerRef} className="h-16 min-w-0 flex-grow">
        <MiniGraph
          newData={timeseries}
          width={width}
          renderValue={renderValue}
        />
      </div>
    </div>
  );
}

export function TimeSeriesValueNumeric(props: Props) {
  return <_TimeSeriesValue {...props} />;
}
