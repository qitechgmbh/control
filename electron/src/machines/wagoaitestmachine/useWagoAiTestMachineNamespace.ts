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
  analogInputs: z.tuple([
    z.number(),
    z.number(),
    z.number(),
    z.number(),
    z.string(),
  ]).optional(),
  wiringErrors: z.tuple([
    z.boolean(),
    z.boolean(),
    z.boolean(),
    z.boolean(),
  ]).optional(),
});

export const stateEventSchema = eventSchema(stateEventDataSchema);

export type StateEvent = z.infer<typeof stateEventDataSchema>;

// ========== Store ==========
export type WagoAiTestMachineNamespaceStore = z.infer<
  typeof stateEventDataSchema
>;

const createMachineStore = (): StoreApi<WagoAiTestMachineNamespaceStore> =>
  create<WagoAiTestMachineNamespaceStore>(() => ({
    measurementRateHz: 1,
    analogInputs: undefined,
    wiringErrors: undefined,
  }));

// ========== Message Handler ==========
function wagoAiTestMachineMessageHandler(
  store: StoreApi<WagoAiTestMachineNamespaceStore>,
): EventHandler {
  return (event) => {
    const oldState = store.getState();

    switch (event.name) {
      case "MeasurementRateHz":
        {
          const newMeasurementRateHz = event.data["MeasurementRateHz"];
          if (
            newMeasurementRateHz &&
            newMeasurementRateHz !== oldState.measurementRateHz
          )
            store.setState({
              ...oldState,
              measurementRateHz: newMeasurementRateHz,
            });
        }
        break;
      case "AnalogInputs":
        {
          const newAnalogInputs = event.data["AnalogInputs"];
          if (newAnalogInputs && newAnalogInputs !== oldState.analogInputs)
            store.setState({
              ...oldState,
              analogInputs: newAnalogInputs,
            });
        }
        break;
      case "WiringErrors":
        {
          const newWiringErrors = event.data["WiringErrors"];
          if (newWiringErrors && newWiringErrors !== oldState.wiringErrors)
            store.setState({
              ...oldState,
              wiringErrors: newWiringErrors,
            });
        }
        break;
    }
  };
}

// ========== Namespace Hook ==========
export const useWagoAiTestMachineNamespace =
  createNamespaceHookImplementation({
    createStore: () => createMachineStore(),
    createEventHandler: wagoAiTestMachineMessageHandler,
  });
