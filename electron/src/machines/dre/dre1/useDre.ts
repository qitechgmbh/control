import { toastError } from "@/components/Toast";
import { useMachineMutate as useMachineMutation } from "@/client/useClient";
import { dre1, MachineIdentificationUnique } from "@/machines/types";
import { dre1SerialRoute } from "@/routes/routes";
import { z } from "zod";
import { useDre1Namespace } from "./dre1Namespace";
import { useMemo } from "react";


function useDre(machine_identification_unique: MachineIdentificationUnique) {

    const schemaTargetDiameter = z.object({ TargetSetTargetDiameter: z.number() });
    const { request: requestTargetDiameter } = useMachineMutation(schemaTargetDiameter);
    const dreSetTargetDiameter = async (target_diameter: number) => {
        requestTargetDiameter({
            machine_identification_unique,
            data: {
                TargetSetTargetDiameter: target_diameter,
            },
        });
    };

    const schemaLowerTolerance = z.object({ TargetSetLowerTolerance: z.number() });
    const { request: requestLowerTolerance } = useMachineMutation(schemaLowerTolerance);
    const dreSetLowerTolerance = async (lower_tolerance: number) => {
        requestLowerTolerance({
            machine_identification_unique,
            data: {
                TargetSetLowerTolerance: lower_tolerance,
            },
        });
    };

    const schemaHigherTolerance = z.object({ TargetSetHigherTolerance: z.number() });
    const { request: requestHigherTolerance } = useMachineMutation(schemaHigherTolerance);
    const dreSetHigherTolerance = async (higher_tolerance: number) => {
        requestHigherTolerance({
            machine_identification_unique,
            data: {
                TargetSetHigherTolerance: higher_tolerance,
            },
        });
    };

    // Read Path
    const { dreDiameter, dreState } = useDre1Namespace(
        machine_identification_unique,
    );

    return { dreDiameter, dreState, dreSetTargetDiameter, dreSetLowerTolerance, dreSetHigherTolerance };
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
