import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Hex } from "@/components/Value";
import { useClient } from "@/client/useClient";
import {
  filterAllowedDevices,
  getMachineProperties,
  machineProperties,
  VENDOR_QITECH,
} from "@/machines/properties";
import React, { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { z } from "zod";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useFormValues } from "@/lib/useFormValues";
import { DeviceRoleComponent } from "@/components/DeviceRole";
import { Alert } from "@/components/Alert";
import { Separator } from "@/components/ui/separator";
import { Icon } from "@/components/Icon";
import { LoadingSpinner } from "@/components/LoadingSpinner";
import { toast } from "sonner";
import { Toast } from "@/components/Toast";
import { EthercatDevicesEventData } from "@/client/mainNamespace";
import { restartBackend } from "@/helpers/troubleshoot_helpers";

type Device = NonNullable<EthercatDevicesEventData["Done"]>["devices"][number];

type Props = {
  device: Device;
};

const formSchema = z.object({
  machine: z
    .string()
    .refine((v) => parseInt(v) < 0xffff, { error: "Value too big" }),
  serial: z
    .string()
    .refine((v) => parseInt(v) < 0xffff, { error: "Value too big" }),
  role: z
    .string()
    .refine((v) => parseInt(v) < 0xffff, { error: "Value too big" }),
});

type FormSchema = z.infer<typeof formSchema>;

export function DeviceEepromDialog({ device }: Props) {
  const [open, setOpen] = React.useState(false);
  const key = useMemo(() => Math.random(), [open]);
  const onClose = () => setOpen(false);

  return (
    <Dialog open={open} onOpenChange={setOpen} modal>
      <DialogTrigger asChild>
        <Button variant="outline">
          <Icon name="lu:Pencil" />
          Assign
        </Button>
      </DialogTrigger>
      <DeviceEepromDialogContent device={device} key={key} setOpen={onClose} />
    </Dialog>
  );
}

// (Empty comment block removed)
type ContentProps = {
  device: Device;
  setOpen: () => void;
};

export function DeviceEepromDialogContent({ device, setOpen }: ContentProps) {
  const client = useClient();
  const [isApplying, setIsApplying] = useState(false);
  const [writeSuccess, setWriteSuccess] = useState(false);

  const [numpadOpen, setNumpadOpen] = useState(false);
  const [numpadPosition, setNumpadPosition] = useState({ left: 0, top: 0 });
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const serialInputRef = useRef<HTMLInputElement | null>(null);
  const serialContainerRef = useRef<HTMLDivElement | null>(null);
  const numpadRef = useRef<HTMLDivElement | null>(null);

  const initialMachine = useMemo(
    () =>
      device.device_identification.device_machine_identification?.machine_identification_unique.machine_identification.machine.toString(),
    [device],
  );

  const form = useForm<FormSchema>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      machine: initialMachine ?? "",
      serial:
        device.device_identification.device_machine_identification?.machine_identification_unique.serial.toString(),
      role: device.device_identification.device_machine_identification?.role.toString(),
    },
    mode: "all",
  });
  const values = useFormValues(form);

  const isChangingMachine = initialMachine != null && values.machine !== initialMachine;

  const performWrite = (values: FormSchema) =>
    client.writeMachineDeviceIdentification({
      hardware_identification_ethercat: {
        subdevice_index:
          device.device_identification.device_hardware_identification.Ethercat!.subdevice_index,
      },
      device_machine_identification: {
        machine_identification_unique: {
          machine_identification: {
            vendor: VENDOR_QITECH,
            machine: parseInt(values.machine!),
          },
          serial: parseInt(values.serial!),
        },
        role: parseInt(values.role!),
      },
    });

  const confirmIfChangingMachine = (): boolean => {
    if (!isChangingMachine) return true;
    return window.confirm(
      "Changing this device to another machine will disconnect it from the current setup. A backend restart is required and terminals may need to be rediscovered (Setup → Troubleshoot → Restart backend). Continue?",
    );
  };

  const onSubmit = (values: FormSchema) => {
    if (!confirmIfChangingMachine()) return;
    performWrite(values).then((res) => {
      if (res.success) {
        setWriteSuccess(true);
        toast(
          <Toast title="Saved" icon="lu:CircleCheck">
            Saved successfully. Restart required to apply changes.
          </Toast>,
        );
      }
    });
  };

  // Apply & restart: always save first; if save fails, block restart and show error
  const handleApplyAndRestart = () => {
    if (!confirmIfChangingMachine()) return;
    form.handleSubmit((values) => {
      setIsApplying(true);
      performWrite(values)
        .then(async (res) => {
          if (!res.success) {
            toast(
              <Toast title="Save failed" icon="lu:CircleAlert">
                Could not save assignment. Restart aborted.
              </Toast>,
            );
            return;
          }
          setWriteSuccess(true);
          toast(
            <Toast title="Saved" icon="lu:CircleCheck">
              Saved. Restarting backend…
            </Toast>,
          );
          const result = await restartBackend();
          if (result.success) {
            toast(
              <Toast title="Backend restart" icon="lu:RotateCcw">
                Backend restart initiated.
              </Toast>,
            );
            setOpen();
          } else {
            toast(
              <Toast title="Backend restart failed" icon="lu:CircleAlert">
                {result.error ?? "Unknown error"}
              </Toast>,
            );
          }
        })
        .finally(() => setIsApplying(false));
    })();
  };

  const updateNumpadPosition = useCallback(() => {
    if (!numpadOpen || !dialogRef.current) return;
    const rect = dialogRef.current.getBoundingClientRect();
    setNumpadPosition({
      left: rect.right + 20,
      top: rect.top + rect.height / 2,
    });
  }, [numpadOpen]);

  const machinePreset = useMemo(() => {
    if (!values.machine) return;
    return getMachineProperties({
      vendor: VENDOR_QITECH,
      machine: parseInt(values.machine),
    });
  }, [values.machine]);

  const filteredAllowedDevices = useMemo(
    () =>
      filterAllowedDevices(
        device.vendor_id,
        device.product_id,
        device.revision,
        machinePreset?.device_roles,
      ),
    [device.product_id, device.revision, machinePreset],
  );

  // if there is only one allowed role, set the role to that immediately
  useEffect(() => {
    const allowedRoles = filteredAllowedDevices.reduce(
      (acc, isAllowed) => acc + (isAllowed ? 1 : 0),
      0,
    );
    if (allowedRoles === 1) {
      // find the index of the first true value
      const index = filteredAllowedDevices.findIndex(
        (isAllowed) => isAllowed === true,
      );
      // set the device role of index
      form.setValue("role", index.toString());
    }
  }, [filteredAllowedDevices]);

  // Position numpad once when it opens
  useEffect(() => {
    updateNumpadPosition();
  }, [numpadOpen]);

  // Close numpad when clicking outside input/numpad
  useEffect(() => {
    if (!numpadOpen) return;

    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target as Node | null;
      const insideSerial = serialContainerRef.current?.contains(target);
      const insideNumpad = numpadRef.current?.contains(target);
      if (!insideSerial && !insideNumpad) {
        setNumpadOpen(false);
      }
    };

    document.addEventListener("pointerdown", handlePointerDown, true);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown, true);
    };
  }, [numpadOpen]);

  // Keep focus on input field when numpad is opened
  useEffect(() => {
    if (numpadOpen && serialInputRef.current) {
      // Use setTimeout to ensure this runs after any other focus changes
      setTimeout(() => {
        if (serialInputRef.current) {
          serialInputRef.current.focus();
        }
      }, 0);
    }
  }, [numpadOpen]);

  // Update numpad position whenever the window resizes
  useEffect(() => {
    window.addEventListener("resize", updateNumpadPosition);
    return () => {
      window.removeEventListener("resize", updateNumpadPosition);
    };
  }, [updateNumpadPosition]);

  // Numpad handlers for serial input
  const numpadHandlers = useMemo(() => {
    const ensureFocus = () => {
      if (
        serialInputRef.current &&
        document.activeElement !== serialInputRef.current
      ) {
        serialInputRef.current.focus();
      }
    };

    const updateCursorPosition = (position: number) => {
      setTimeout(() => {
        if (serialInputRef.current) {
          serialInputRef.current.setSelectionRange(position, position);
        }
      }, 0);
    };

    const getCurrentValue = () => {
      return form.getValues("serial") || "";
    };

    return {
      appendDigit: (digit: string) => {
        if (!serialInputRef.current) return;

        ensureFocus();
        const input = serialInputRef.current;
        const start = input.selectionStart || 0;
        const end = input.selectionEnd || 0;
        const currentValue = getCurrentValue();
        const newValue =
          currentValue.slice(0, start) + digit + currentValue.slice(end);

        form.setValue("serial", newValue, { shouldValidate: true });
        updateCursorPosition(start + 1);
      },

      addDecimal: () => {
        // Not needed for serial (U16 integer), but keeping for consistency
        // Could be used for other numeric inputs if needed
      },

      deleteChar: () => {
        if (!serialInputRef.current) return;

        ensureFocus();
        const input = serialInputRef.current;
        const start = input.selectionStart || 0;
        const end = input.selectionEnd || 0;
        const currentValue = getCurrentValue();

        let newValue: string;
        let newPosition: number;

        if (start !== end) {
          // Delete selection
          newValue = currentValue.slice(0, start) + currentValue.slice(end);
          newPosition = start;
        } else if (start > 0) {
          // Backspace
          newValue =
            currentValue.slice(0, start - 1) + currentValue.slice(start);
          newPosition = start - 1;
        } else {
          return;
        }

        form.setValue("serial", newValue, { shouldValidate: true });
        updateCursorPosition(newPosition);
      },

      toggleSign: () => {
        // Not needed for U16 (unsigned), but keeping for consistency
      },

      moveCursorLeft: () => {
        if (!serialInputRef.current) return;

        ensureFocus();
        const currentPos = serialInputRef.current.selectionStart || 0;
        if (currentPos > 0) {
          serialInputRef.current.setSelectionRange(
            currentPos - 1,
            currentPos - 1,
          );
        }
      },

      moveCursorRight: () => {
        if (!serialInputRef.current) return;

        ensureFocus();
        const currentPos = serialInputRef.current.selectionStart || 0;
        const currentValue = getCurrentValue();
        if (currentPos < currentValue.length) {
          serialInputRef.current.setSelectionRange(
            currentPos + 1,
            currentPos + 1,
          );
        }
      },
    };
  }, [form]);
  return (
    <>
      <DialogContent
        // Keep dialog open on any outside interaction; closing is manual via controls
        onInteractOutside={(e) => e.preventDefault()}
        onPointerDownOutside={(e) => e.preventDefault()}
        onEscapeKeyDown={(e) => e.preventDefault()}
      >
        <DialogHeader>
          <DialogTitle>Machine Assignment</DialogTitle>
          <p>
            for {device.name}
            <Hex value={device.configured_address} />
          </p>
          <DialogDescription>
            To assign the device to a machine, select the machine, serial number
            & device role.
          </DialogDescription>
        </DialogHeader>
        <Separator />
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            {/* machine type dropdown */}
            <FormField
              control={form.control}
              name="machine"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Maschine</FormLabel>
                  <FormControl>
                    <Select {...field} onValueChange={field.onChange}>
                      <SelectTrigger className="w-min">
                        <SelectValue placeholder="Machine" />
                      </SelectTrigger>
                      <SelectContent>
                        {machineProperties.map((machine) => (
                          <SelectItem
                            key={machine.machine_identification.machine}
                            value={machine.machine_identification.machine.toString()}
                          >
                            {machine.name} {machine.version}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            {/* Serial Number */}
            <FormField
              control={form.control}
              name="serial"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Serial</FormLabel>
                  <FormControl>
                    <Input {...field} placeholder="1234" />
                  </FormControl>
                  <FormDescription>
                    Serial number of the machine.
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
            {/* Device Role */}
            <FormField
              control={form.control}
              name="role"
              disabled={!machinePreset}
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Device Role</FormLabel>
                  <FormControl>
                    <Select {...field}>
                      <SelectTrigger className="w-min">
                        <SelectValue placeholder="Device Role" />
                      </SelectTrigger>
                      <SelectContent>
                        {machinePreset?.device_roles.map((device_role, i) => (
                          <SelectItem
                            key={device_role.role}
                            value={device_role.role.toString()}
                            disabled={!filteredAllowedDevices[i]}
                          >
                            <DeviceRoleComponent device_role={device_role} />
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <Separator />
            {isChangingMachine && (
              <Alert title="Changing machine assignment" variant="warning">
                This will disconnect the device from its current machine. Restart
                required; if terminals disappear, use Setup → Troubleshoot →
                Restart backend to rediscover.
              </Alert>
            )}
            {form.formState.isDirty && !writeSuccess && (
              <p className="text-muted-foreground text-sm">
                Save or Apply & restart for assignment changes to apply.
              </p>
            )}
            <div className="flex flex-wrap items-center gap-2">
              <Button
                type="submit"
                disabled={!form.formState.isValid || isApplying}
                onClick={() => setWriteSuccess(false)}
              >
                <Icon name="lu:Save" /> Save
              </Button>
              <Button
                type="button"
                variant="outline"
                disabled={!form.formState.isValid || isApplying}
                onClick={handleApplyAndRestart}
                aria-busy={isApplying}
                title="Saves assignment then restarts the backend. Restart is required for changes to take effect."
              >
                {isApplying ? (
                  <>
                    <LoadingSpinner />
                    Saving & restarting…
                  </>
                ) : (
                  <>
                    <Icon name="lu:RotateCcw" />
                    Apply & restart
                  </>
                )}
              </Button>
              {writeSuccess && (
                <Button type="button" variant="secondary" onClick={() => setOpen()}>
                  Close
                </Button>
              )}
            </div>
            <Alert title="Restart required" variant="info">
              The backend must be restarted for assignment changes to take
              effect.
            </Alert>
          </form>
        </Form>
      </DialogContent>
    </>
  );
}
