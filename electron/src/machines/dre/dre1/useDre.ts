import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { dre1, MachineIdentificationUnique } from "@/machines/types";
import { dre1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useDre1Namespace } from "./dre1Namespace";
import { useEffect, useMemo } from "react";
import { useStateOptimistic } from "@/lib/useStateOptimistic";


function useDre(machine_identification_unique: MachineIdentificationUnique) {
    // Write Path
    const dreStateOptimistic = useStateOptimistic<{
        target_diameter: number;
        lower_tolerance: number;
        higher_tolerance: number;
    }>();
    const schemaTargetDiameter = z.object({ TargetSetTargetDiameter: z.number() });
    const { request: requestTargetDiameter } = useMachineMutation(schemaTargetDiameter);
    const dreSetTargetDiameter = async (target_diameter: number) => {
        if (dreStateOptimistic.value) {
            dreStateOptimistic.setOptimistic({
                ...dreStateOptimistic.value,
                target_diameter: target_diameter,
            });
        }
        requestTargetDiameter({
            machine_identification_unique,
            data: {
                TargetSetTargetDiameter: target_diameter,
            },
        }).then((response) => {
            if (!response.success) dreStateOptimistic.resetToReal();
        })
            .catch(() => dreStateOptimistic.resetToReal());;
    };

    const schemaLowerTolerance = z.object({ TargetSetLowerTolerance: z.number() });
    const { request: requestLowerTolerance } = useMachineMutation(schemaLowerTolerance);
    const dreSetLowerTolerance = async (lower_tolerance: number) => {
        if (dreStateOptimistic.value) {
            dreStateOptimistic.setOptimistic({
                ...dreStateOptimistic.value,
                lower_tolerance: lower_tolerance,
            });
        }
        requestLowerTolerance({
            machine_identification_unique,
            data: {
                TargetSetLowerTolerance: lower_tolerance,
            },
        }).then((response) => {
            if (!response.success) dreStateOptimistic.resetToReal();
        })
            .catch(() => dreStateOptimistic.resetToReal());;
    };

    const schemaHigherTolerance = z.object({ TargetSetHigherTolerance: z.number() });
    const { request: requestHigherTolerance } = useMachineMutation(schemaHigherTolerance);
    const dreSetHigherTolerance = async (higher_tolerance: number) => {
        if (dreStateOptimistic.value) {
            dreStateOptimistic.setOptimistic({
                ...dreStateOptimistic.value,
                higher_tolerance: higher_tolerance,
            });
        }
        requestHigherTolerance({
            machine_identification_unique,
            data: {
                TargetSetHigherTolerance: higher_tolerance,
            },
        }).then((response) => {
            if (!response.success) dreStateOptimistic.resetToReal();
        })
            .catch(() => dreStateOptimistic.resetToReal());;
    };

    // Read Path
    const { dreDiameter, dreState } = useDre1Namespace(
        machine_identification_unique,
    );

    // Update real values from server
    useEffect(() => {
        if (dreState?.data) {
            dreStateOptimistic.setReal(dreState.data);
        }
    }, [dreState]);

    return {
        dreDiameter,
        dreState,
        dreSetTargetDiameter,
        dreSetLowerTolerance,
        dreSetHigherTolerance,
        dreStateIsLoading:
            dreStateOptimistic.isOptimistic ||
            !dreStateOptimistic.isInitialized,
        dreStateIsDisabled:
            dreStateOptimistic.isOptimistic ||
            !dreStateOptimistic.isInitialized,
    };
}


export function useDre1() {
    const { serial: serialString } = dre1SerialRoute.useParams();

    // Memoize the machine identification to keep it stable between renders
    const machineIdentification: MachineIdentificationUnique = useMemo(() => {
        const serial = parseInt(serialString); // Use 0 as fallback if NaN

        if (isNaN(serial)) {
            toastError(
                "Invalid Serial Number",
                `"${serialString}" is not a valid serial number.`,
            );

            return {
                machine_identification: {
                    vendor: 0,
                    machine: 0,
                },
                serial: 0,
            };
        }

        return {
            machine_identification: dre1.machine_identification,
            serial,
        };
    }, [serialString]); // Only recreate when serialString changes

    const dre = useDre(machineIdentification);


    return {
        ...dre,
    };
}
