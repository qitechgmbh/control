import React, { useEffect, useRef, useState } from "react";
import uPlot from "uplot";
import "uplot/dist/uPlot.min.css";
import { TimeSeries, seriesToUPlotData, getSeriesMinMax } from "@/lib/timeseries";
import { renderUnitSymbol, Unit } from "@/control/units";
import * as XLSX from 'xlsx';

type BigGraphProps = {
    newData: TimeSeries | null;
    threshold1: number;
    threshold2: number;
    target: number;
    unit?: Unit;
    renderValue?: (value: number) => string;
};

// Time window options
const TIME_WINDOW_OPTIONS = [
    { value: 10 * 1000, label: '10 sec' },
    { value: 30 * 1000, label: '30 sec' },
    { value: 1 * 60 * 1000, label: '1 min' },
    { value: 5 * 60 * 1000, label: '5 min' },
    { value: 10 * 60 * 1000, label: '10 min' },
];

export function BigGraph({
    newData,
    threshold1,
    threshold2,
    target,
    unit,
    renderValue,
}: BigGraphProps) {
    const containerRef = useRef<HTMLDivElement | null>(null);
    const uplotRef = useRef<uPlot | null>(null);
    const [viewMode, setViewMode] = useState<'default' | 'all' | 'manual'>('default');
    const [selectedTimeWindow, setSelectedTimeWindow] = useState<number>(1 * 60 * 1000); // Default to 1 minute
    const [cursorValue, setCursorValue] = useState<number | null>(null);
    const startTimeRef = useRef<number | null>(null);
    const manualScaleRef = useRef<{ x: { min: number, max: number }, y: { min: number, max: number } } | null>(null);
    const isUserZoomingRef = useRef(false);
    const UPDATE_INTERVAL_MS = newData?.long?.sampleInterval ?? 100;

    // Excel export function
    const exportToExcel = () => {
        try {
            if (!newData?.long) {
                alert('No data to export');
                return;
            }

            // Get data from long buffer
            const [timestamps, values] = seriesToUPlotData(newData.long);

            if (timestamps.length === 0) {
                alert('No data to export');
                return;
            }

            // Prepare data for Excel export
            const exportData = timestamps.map((timestamp, index) => ({
                'Timestamp': new Date(timestamp).toLocaleString(),
                [`Value (${renderUnitSymbol(unit)})`]: renderValue ? renderValue(values[index]) : values[index]?.toFixed(3) || '',
                'Within Tolerance': (values[index] >= threshold2 && values[index] <= threshold1) ? 'Yes' : 'No',
                'Deviation from Target': renderValue ? renderValue(values[index] - target) : (values[index] - target).toFixed(3)
            }));

            // Create workbook and worksheet
            const workbook = XLSX.utils.book_new();
            const worksheet = XLSX.utils.json_to_sheet(exportData);

            // Create summary data
            const summaryData = [
                ['Graph Data Export Summary', ''],
                ['Export Date', new Date().toLocaleString()],
                ['', ''],
                ['Parameters', ''],
                [`Target (${renderUnitSymbol(unit)})`, renderValue ? renderValue(target) : target.toFixed(3)],
                [`Upper Threshold (${renderUnitSymbol(unit)})`, renderValue ? renderValue(threshold1) : threshold1.toFixed(3)],
                [`Lower Threshold (${renderUnitSymbol(unit)})`, renderValue ? renderValue(threshold2) : threshold2.toFixed(3)],
                [`Tolerance Range (${renderUnitSymbol(unit)})`, renderValue ? renderValue(threshold1 - threshold2) : (threshold1 - threshold2).toFixed(3)],
                ['', ''],
                ['Statistics', ''],
                ['Total Data Points', timestamps.length],
                ['Time Range Start', new Date(timestamps[0]).toLocaleString()],
                ['Time Range End', new Date(timestamps[timestamps.length - 1]).toLocaleString()],
                [`Min Value (${renderUnitSymbol(unit)})`, renderValue ? renderValue(Math.min(...values)) : Math.min(...values).toFixed(3)],
                [`Max Value (${renderUnitSymbol(unit)})`, renderValue ? renderValue(Math.max(...values)) : Math.max(...values).toFixed(3)],
                [`Average Value (${renderUnitSymbol(unit)})`, renderValue ? renderValue(values.reduce((a, b) => a + b, 0) / values.length) : (values.reduce((a, b) => a + b, 0) / values.length).toFixed(3)],
                ['Points Within Tolerance', values.filter(v => v >= threshold2 && v <= threshold1).length],
                ['Points Outside Tolerance', values.filter(v => v < threshold2 || v > threshold1).length],
            ];

            // Create summary worksheet
            const summaryWorksheet = XLSX.utils.aoa_to_sheet(summaryData);

            // Add worksheets to workbook
            XLSX.utils.book_append_sheet(workbook, summaryWorksheet, 'Summary');
            XLSX.utils.book_append_sheet(workbook, worksheet, 'Data');

            // Generate filename with timestamp
            const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
            const filename = `graph_data_${timestamp}.xlsx`;

            // Save file
            XLSX.writeFile(workbook, filename);

            console.log(`Excel file exported: ${filename}`);
        } catch (error) {
            console.error('Error exporting to Excel:', error);
            alert('Error exporting data to Excel. Please try again.');
        }
    };

    // Main useEffect for chart creation
    useEffect(() => {
        if (!containerRef.current || !newData?.long?.timeWindow) return;

        // Get data from long buffer
        const [timestamps, values] = seriesToUPlotData(newData.long);

        // Set start time to the first timestamp if not already set
        if (timestamps.length > 0 && startTimeRef.current === null) {
            startTimeRef.current = timestamps[0];
        }

        const { min: minY, max: maxY } = getSeriesMinMax(newData.long);
        const range = maxY - minY || 1;

        const makeThresholdLine = (y: number) => timestamps.map(() => y);

        const uData: uPlot.AlignedData = [
            timestamps,
            values,
            makeThresholdLine(threshold1),
            makeThresholdLine(threshold2),
            makeThresholdLine(target),
        ];

        // Calculate view based on current mode
        const lastTimestamp = timestamps[timestamps.length - 1] ?? 0;
        const defaultViewStart = Math.max(lastTimestamp - selectedTimeWindow, startTimeRef.current ?? 0);
        const fullStart = startTimeRef.current ?? timestamps[0] ?? 0;

        let initialMin, initialMax;
        if (viewMode === 'manual' && manualScaleRef.current) {
            initialMin = manualScaleRef.current.x.min;
            initialMax = manualScaleRef.current.x.max;
        } else if (viewMode === 'default') {
            initialMin = defaultViewStart;
            initialMax = lastTimestamp;
        } else {
            initialMin = fullStart;
            initialMax = lastTimestamp;
        }

        const createChart = (width: number, height: number) =>
            new uPlot(
                {
                    width,
                    height,
                    padding: [20, 60, 40, 50],
                    cursor: {
                        show: true,
                        x: true,
                        y: true,
                        drag: { x: true, y: true, setScale: true },
                        sync: {
                            key: "myCursor",
                        },
                    },
                    legend: {
                        show: true,
                        live: true,
                    },
                    hooks: {
                        setScale: [
                            (u) => {
                                // Only switch to manual mode if this is a user-initiated zoom
                                if (isUserZoomingRef.current) {
                                    const xScale = u.scales.x;
                                    const yScale = u.scales.y;
                                    manualScaleRef.current = {
                                        x: { min: xScale.min!, max: xScale.max! },
                                        y: { min: yScale.min!, max: yScale.max! }
                                    };
                                    setViewMode('manual');
                                    isUserZoomingRef.current = false;
                                }
                            },
                        ],
                        setCursor: [
                            (u) => {
                                // Update cursor value when cursor moves
                                if (typeof u.cursor.idx === 'number' && u.data[1][u.cursor.idx] !== undefined) {
                                    const timestamp = u.cursor.idx !== null && u.cursor.idx !== undefined ? u.data[0][u.cursor.idx] : undefined;
                                    const value = u.cursor.idx !== null && u.cursor.idx !== undefined ? u.data[1][u.cursor.idx] : undefined;
                                    const cur = newData?.current;

                                    // Check if cursor is near current time (within 1 second = 1000ms)
                                    const isNearCurrent = cur && timestamp !== undefined && Math.abs(timestamp - cur.timestamp) < 1000;

                                    // Use current value if near current time, otherwise use historical data
                                    const displayValue = isNearCurrent ? cur.value : value;
                                    setCursorValue(displayValue ?? null);
                                } else {
                                    setCursorValue(null);
                                }
                            },
                        ],
                    },
                    scales: {
                        x: {
                            time: true,
                            min: initialMin,
                            max: initialMax
                        },
                        y: {
                            auto: true,
                            min: viewMode === 'manual' && manualScaleRef.current ? manualScaleRef.current.y.min : minY - range * 0.1,
                            max: viewMode === 'manual' && manualScaleRef.current ? manualScaleRef.current.y.max : maxY + range * 0.1,
                        },
                    },
                    axes: [
                        {
                            stroke: "#333",
                            label: "Time",
                            labelSize: 18,
                            grid: { stroke: "#eee", width: 1 },
                            values: (u, ticks) =>
                                ticks.map((ts) =>
                                    new Date(ts).toLocaleTimeString("en-GB", {
                                        hour12: false,
                                        hour: "2-digit",
                                        minute: "2-digit",
                                        second: "2-digit",
                                    })
                                ),
                        },
                        {
                            stroke: "#333",
                            label: "Value",
                            labelSize: 18,
                            grid: { stroke: "#eee", width: 1 },
                            values: (u, ticks) => ticks.map((v) => renderValue ? renderValue(v) : v.toFixed(3)),
                        },
                    ],
                    series: [
                        { label: "Time" },
                        {
                            label: "Value",
                            stroke: "#007aff",
                            width: 2,
                            spanGaps: true,
                        },
                        {
                            label: "Threshold 1",
                            stroke: "red",
                            width: 1,
                            dash: [5, 5],
                            show: true,
                        },
                        {
                            label: "Threshold 2",
                            stroke: "orange",
                            width: 1,
                            dash: [5, 5],
                            show: true,
                        },
                        {
                            label: "Target",
                            stroke: "#aaa",
                            width: 1,
                            show: true,
                        },
                    ],
                },
                uData,
                containerRef.current!
            );

        // Set up event listeners for detecting user interaction
        const handleMouseDown = () => {
            isUserZoomingRef.current = true;
        };

        const handleWheel = () => {
            isUserZoomingRef.current = true;
        };

        const resizeObserver = new ResizeObserver((entries) => {
            for (const entry of entries) {
                const { width, height } = entry.contentRect;
                if (uplotRef.current) {
                    uplotRef.current.setSize({ width, height });
                } else {
                    uplotRef.current = createChart(width, height);
                    // Add event listeners after chart creation
                    if (containerRef.current) {
                        containerRef.current.addEventListener('mousedown', handleMouseDown);
                        containerRef.current.addEventListener('wheel', handleWheel);
                    }
                }
            }
        });

        resizeObserver.observe(containerRef.current);

        return () => {
            resizeObserver.disconnect();
            if (containerRef.current) {
                containerRef.current.removeEventListener('mousedown', handleMouseDown);
                containerRef.current.removeEventListener('wheel', handleWheel);
            }
            uplotRef.current?.destroy();
            uplotRef.current = null;
        };
    }, [
        newData?.long?.timeWindow,
        threshold1,
        threshold2,
        target,
        newData?.long?.sampleInterval,
        viewMode,
        selectedTimeWindow,
    ]);

    // Live updates useEffect
    useEffect(() => {
        // This effect handles live updates without recreating the chart
        if (!uplotRef.current || !newData?.current) return;

        const updateLiveData = () => {
            if (!newData?.long || !newData?.current || !uplotRef.current) return;

            // Get updated data from long buffer
            const [timestamps, values] = seriesToUPlotData(newData.long);

            // Add current value for live updates if it's newer than the last long buffer entry
            const cur = newData.current;
            const liveTimestamps = [...timestamps];
            const liveValues = [...values];

            // Only add current value if it's significantly newer than the last data point
            const lastTimestamp = timestamps[timestamps.length - 1] || 0;
            const timeSinceLastPoint = cur.timestamp - lastTimestamp;

            // Add current point if it's newer than the last point by at least half the update interval
            if (timeSinceLastPoint > UPDATE_INTERVAL_MS * 0.5) {
                liveTimestamps.push(cur.timestamp);
                liveValues.push(cur.value);
            }

            if (liveTimestamps.length === 0) return;

            // Set start time if this is the first data point
            if (startTimeRef.current === null) {
                startTimeRef.current = liveTimestamps[0];
            }

            // Calculate min/max including the live current value
            const minY = Math.min(...liveValues);
            const maxY = Math.max(...liveValues);
            const range = maxY - minY || 1;

            const thresholdSeries1 = liveTimestamps.map(() => threshold1);
            const thresholdSeries2 = liveTimestamps.map(() => threshold2);
            const targetSeries = liveTimestamps.map(() => target);

            // Store current scale BEFORE setData if in manual mode
            let preservedScale: { x: { min: number; max: number }; y: { min: number; max: number } } | null = null;
            if (viewMode === 'manual') {
                preservedScale = {
                    x: { min: uplotRef.current.scales.x.min!, max: uplotRef.current.scales.x.max! },
                    y: { min: uplotRef.current.scales.y.min!, max: uplotRef.current.scales.y.max! }
                };
            }

            uplotRef.current.setData([
                liveTimestamps,
                liveValues,
                thresholdSeries1,
                thresholdSeries2,
                targetSeries,
            ]);

            // Get the latest timestamp from the actual data (including live current value)
            const latestTimestamp = liveTimestamps[liveTimestamps.length - 1];

            // Update scales based on view mode
            if (viewMode === 'default') {
                // Default mode: show selected time window, properly aligned to current time
                const defaultViewStart = latestTimestamp - selectedTimeWindow;

                // Filter values to only include those within the selected time window
                const recentIndices = liveTimestamps
                    .map((timestamp, index) => ({ timestamp, index }))
                    .filter(({ timestamp }) => timestamp >= defaultViewStart)
                    .map(({ index }) => index);

                const recentValues = recentIndices.map(index => liveValues[index]);

                // Calculate Y range based only on recent data
                const recentMinY = recentValues.length > 0 ? Math.min(...recentValues) : minY;
                const recentMaxY = recentValues.length > 0 ? Math.max(...recentValues) : maxY;
                const recentRange = recentMaxY - recentMinY || 1;

                uplotRef.current.setScale("x", {
                    min: defaultViewStart,
                    max: latestTimestamp,
                });
                uplotRef.current.setScale("y", {
                    min: recentMinY - recentRange * 0.1,
                    max: recentMaxY + recentRange * 0.1,
                });
            } else if (viewMode === 'all') {
                // Show all mode: keep the start time fixed but extend the end time
                const fullStart = startTimeRef.current ?? liveTimestamps[0];
                uplotRef.current.setScale("x", {
                    min: fullStart,
                    max: latestTimestamp,
                });
                uplotRef.current.setScale("y", {
                    min: minY - range * 0.1,
                    max: maxY + range * 0.1,
                });
            } else if (viewMode === 'manual' && preservedScale) {
                // Manual mode: restore the exact scale that was there before setData
                uplotRef.current.setScale("x", {
                    min: preservedScale.x.min,
                    max: preservedScale.x.max,
                });
                uplotRef.current.setScale("y", {
                    min: preservedScale.y.min,
                    max: preservedScale.y.max,
                });
            }
        };

        updateLiveData();
    }, [newData?.current?.timestamp, viewMode, selectedTimeWindow, threshold1, threshold2, target]);

    const setDefaultView = () => {
        setViewMode('default');
        manualScaleRef.current = null; // Clear manual scale

        if (!uplotRef.current || !newData?.long) return;

        // Get data from long buffer
        const [timestamps, values] = seriesToUPlotData(newData.long);

        if (timestamps.length === 0) return;

        // Use the actual latest timestamp from the data array
        const latestTimestamp = timestamps[timestamps.length - 1];
        const defaultViewStart = latestTimestamp - selectedTimeWindow;

        // Filter values to only include those within the selected time window
        const recentIndices = timestamps
            .map((timestamp, index) => ({ timestamp, index }))
            .filter(({ timestamp }) => timestamp >= defaultViewStart)
            .map(({ index }) => index);

        const recentValues = recentIndices.map(index => values[index]);

        // Calculate Y range based only on recent data
        const minY = recentValues.length > 0 ? Math.min(...recentValues) : Math.min(...values);
        const maxY = recentValues.length > 0 ? Math.max(...recentValues) : Math.max(...values);
        const range = maxY - minY || 1;

        uplotRef.current.setScale("x", {
            min: defaultViewStart,
            max: latestTimestamp
        });
        uplotRef.current.setScale("y", {
            min: minY - range * 0.1,
            max: maxY + range * 0.1,
        });
    };

    const setShowAllView = () => {
        setViewMode('all');
        manualScaleRef.current = null; // Clear manual scale

        if (!uplotRef.current || !newData?.long) return;

        // Get data from long buffer
        const [timestamps, values] = seriesToUPlotData(newData.long);

        if (timestamps.length === 0) return;

        const fullStart = startTimeRef.current ?? timestamps[0];
        const fullEnd = timestamps[timestamps.length - 1];
        const minY = Math.min(...values);
        const maxY = Math.max(...values);
        const range = maxY - minY || 1;

        uplotRef.current.setScale("x", {
            min: fullStart,
            max: fullEnd
        });
        uplotRef.current.setScale("y", {
            min: minY - range * 0.1,
            max: maxY + range * 0.1,
        });
    };

    // Handle time window selection change
    const handleTimeWindowChange = (event: React.ChangeEvent<HTMLSelectElement>) => {
        const newTimeWindow = parseInt(event.target.value);
        setSelectedTimeWindow(newTimeWindow);

        // If currently in default view, immediately apply the new time window
        if (viewMode === 'default') {
            setDefaultView();
        }
    };

    // Get the display value: cursor value if hovering, otherwise latest value
    const displayValue = cursorValue !== null ? cursorValue : newData?.current?.value;

    // Get the current time window label for the button
    const currentTimeWindowLabel = TIME_WINDOW_OPTIONS.find(option => option.value === selectedTimeWindow)?.label || '1 min';

    return (
        <div style={{ width: "100%", height: "100%", display: "flex", flexDirection: "column" }}>
            {/* Control Panel - Outside the graph */}
            <div
                style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    padding: "8px",
                    borderBottom: "1px solid #eee",
                    backgroundColor: "#f9f9f9",
                    flexShrink: 0,
                }}
            >
                {/* Current Value Display - now shows cursor value when hovering */}
                <div
                    style={{
                        background: "rgba(255, 255, 255, 0.9)",
                        border: "1px solid #ccc",
                        padding: "6px 12px",
                        borderRadius: "4px",
                        fontSize: "14px",
                        fontWeight: "bold",
                    }}
                >
                    Current: {displayValue !== undefined && displayValue !== null ? (renderValue ? renderValue(displayValue) : displayValue.toFixed(3)) : 'N/A'} {renderUnitSymbol(unit)}
                </div>

                {/* Control Buttons Container */}
                <div style={{ display: "flex", gap: "8px", alignItems: "center" }}>
                    {/* Time Window Selector */}
                    <div style={{ display: "flex", alignItems: "center", gap: "4px" }}>
                        <label style={{ fontSize: "12px", fontWeight: "500" }}>
                            Time:
                        </label>
                        <select
                            value={selectedTimeWindow}
                            onChange={handleTimeWindowChange}
                            style={{
                                ...buttonStyle,
                                backgroundColor: 'white',
                                color: 'black',
                                minWidth: '70px',
                            }}
                        >
                            {TIME_WINDOW_OPTIONS.map(option => (
                                <option key={option.value} value={option.value}>
                                    {option.label}
                                </option>
                            ))}
                        </select>
                    </div>

                    {/* Excel Export Button */}
                    <button
                        onClick={exportToExcel}
                        style={{
                            ...buttonStyle,
                            backgroundColor: '#4CAF50',
                            color: 'white',
                            fontWeight: 'bold',
                        }}
                        onMouseOver={(e) => (e.target as HTMLButtonElement).style.backgroundColor = '#45a049'}
                        onMouseOut={(e) => (e.target as HTMLButtonElement).style.backgroundColor = '#4CAF50'}
                        title="Export graph data to Excel"
                    >
                        📊 Excel
                    </button>

                    {/* View Control Buttons */}
                    <button
                        onClick={setDefaultView}
                        style={{
                            ...buttonStyle,
                            backgroundColor: viewMode === 'default' ? '#007aff' : 'white',
                            color: viewMode === 'default' ? 'white' : 'black',
                        }}
                    >
                        Last {currentTimeWindowLabel}
                    </button>
                    <button
                        onClick={setShowAllView}
                        style={{
                            ...buttonStyle,
                            backgroundColor: viewMode === 'all' ? '#007aff' : 'white',
                            color: viewMode === 'all' ? 'white' : 'black',
                        }}
                    >
                        Show All
                    </button>
                </div>
            </div>

            {/* Graph Container */}
            <div
                ref={containerRef}
                style={{
                    width: "100%",
                    height: "100%",
                    overflow: "hidden",
                    flex: 1,
                }}
            />
        </div>
    );
}

const buttonStyle: React.CSSProperties = {
    border: "1px solid #ccc",
    padding: "6px 12px",
    borderRadius: "4px",
    cursor: "pointer",
    fontSize: "12px",
    fontWeight: "500",
};
