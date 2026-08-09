import { useEffect, useRef, useState } from "react";
import { MachinePhase } from "./MixingMachinePreview";
import { ExtruderLinkState } from "./MixerExtruderLink";

export function useMixerDemo() {
  const [phase, setPhase] = useState<MachinePhase>("idle");
  const [ratioA, setRatioA] = useState(70);
  const [feedRate, setFeedRate] = useState(12);
  const [hopperAEmpty, setHopperAEmpty] = useState(false);
  const [hopperBEmpty, setHopperBEmpty] = useState(false);
  const [hopperALow, setHopperALowState] = useState(false);
  const [hopperBLow, setHopperBLowState] = useState(false);
  const [mixerFault, setMixerFault] = useState(false);
  const [extruderLinkState, setExtruderLinkState] =
    useState<ExtruderLinkState>("ready");
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const ratioB = 100 - ratioA;
  const running = phase === "running";
  const busy = phase === "starting" || phase === "purging";
  const canStart =
    phase === "idle" &&
    !hopperAEmpty &&
    !hopperBEmpty &&
    !mixerFault &&
    extruderLinkState === "ready";

  useEffect(
    () => () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    },
    [],
  );

  useEffect(() => {
    if (mixerFault && phase !== "idle") {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
      setPhase("fault");
    } else if (!mixerFault && phase === "fault") {
      setPhase("idle");
    }
  }, [mixerFault, phase]);

  useEffect(() => {
    if ((hopperAEmpty || hopperBEmpty) && phase === "running") {
      setPhase("purging");
      timeoutRef.current = setTimeout(() => setPhase("idle"), 1200);
    }
  }, [hopperAEmpty, hopperBEmpty, phase]);

  useEffect(() => {
    if (
      extruderLinkState !== "ready" &&
      (phase === "running" || phase === "starting")
    ) {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
      setPhase("purging");
      timeoutRef.current = setTimeout(() => setPhase("idle"), 1200);
    }
  }, [extruderLinkState, phase]);

  const start = () => {
    if (!canStart) return;
    setPhase("starting");
    timeoutRef.current = setTimeout(() => setPhase("running"), 900);
  };

  const stop = () => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    if (phase === "starting") {
      setPhase("idle");
      return;
    }
    setPhase("purging");
    timeoutRef.current = setTimeout(() => setPhase("idle"), 1200);
  };

  const setHopperALow = (low: boolean) => {
    setHopperALowState(low);
    if (low) setHopperAEmpty(false);
  };

  const setHopperBLow = (low: boolean) => {
    setHopperBLowState(low);
    if (low) setHopperBEmpty(false);
  };

  const setHopperAEmptyState = (empty: boolean) => {
    setHopperAEmpty(empty);
    if (empty) setHopperALowState(false);
  };

  const setHopperBEmptyState = (empty: boolean) => {
    setHopperBEmpty(empty);
    if (empty) setHopperBLowState(false);
  };

  const reset = () => {
    if (timeoutRef.current) clearTimeout(timeoutRef.current);
    setPhase("idle");
    setHopperAEmpty(false);
    setHopperBEmpty(false);
    setHopperALowState(false);
    setHopperBLowState(false);
    setMixerFault(false);
    setExtruderLinkState("ready");
  };

  return {
    phase,
    ratioA,
    ratioB,
    feedRate,
    hopperAEmpty,
    hopperBEmpty,
    hopperALow,
    hopperBLow,
    mixerFault,
    extruderLinkState,
    running,
    busy,
    canStart,
    setRatioA,
    setFeedRate,
    setHopperAEmpty: setHopperAEmptyState,
    setHopperBEmpty: setHopperBEmptyState,
    setHopperALow,
    setHopperBLow,
    setMixerFault,
    setExtruderLinkState,
    start,
    stop,
    reset,
  };
}
