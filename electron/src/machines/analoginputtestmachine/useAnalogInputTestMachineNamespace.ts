import {
  createNamespaceHookImplementation,
  EventHandler,
  eventSchema,
} from "@/client/socketioStore";
import z from "zod";
import { create, StoreApi } from "zustand";

// ========== Event Schema ==========

export const stateEventDataSchema = z.object({
  measurementRateHz: z.number().optional(),
  currentMeasurement: z.tuple([z.number(), z.string()]).optional(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

// ========== Store ==========
export type AnalogInputTestMachineNamespaceStore = z.infer<
  typeof stateEventDataSchema
>;

const createMachineStore = (): StoreApi<AnalogInputTestMachineNamespaceStore> =>
  create<AnalogInputTestMachineNamespaceStore>(() => ({
    measurementRateHz: 1,
    currentMeasurement: undefined,
  }));

// ========== Message Handler ==========
function analogInputTestMachineMessageHandler(
  store: StoreApi<AnalogInputTestMachineNamespaceStore>,
): EventHandler {
  return (event) => {
    const oldState = store.getState();
    const newMeasurementDataHz = event.data["MeasurementRateHz"];
    const newMeasurement: [number, string] = event.data["Measurement"];

    switch (event.name) {
      case "MeasurementRateHz":
        if (
          newMeasurementDataHz &&
          newMeasurementDataHz !== oldState.measurementRateHz
        )
          store.setState({
            ...oldState,
            measurementRateHz: newMeasurementDataHz,
          });
        break;
      case "Measurement":
        if (newMeasurement && newMeasurement !== oldState.currentMeasurement)
          store.setState({
            ...oldState,
            currentMeasurement: newMeasurement,
          });
        break;
    }
  };
}

// ========== Namespace Hook ==========
export const useAnalogInputTestMachineNamespace =
  createNamespaceHookImplementation({
    createStore: () => createMachineStore(),
    createEventHandler: analogInputTestMachineMessageHandler,
  });
