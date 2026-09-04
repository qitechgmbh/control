import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { useClient } from "@/client/useClient";
import { machineProperties } from "@/machines/properties";
import React, { useEffect, useMemo, useRef, useState } from "react";
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
import { Alert } from "@/components/Alert";
import { Separator } from "@/components/ui/separator";
import { Icon } from "@/components/Icon";
import { LoadingSpinner } from "@/components/LoadingSpinner";
import { toast } from "sonner";
import { Toast } from "@/components/Toast";
import { ModbusDevice } from "@/client/mainNamespace";
import { restartBackend } from "@/helpers/troubleshoot_helpers";
import { TouchNumpad } from "@/components/touch/TouchNumpad";

const VENDOR_QITECH = 0x0001;

type Props = {
  device: ModbusDevice;
};

const formSchema = z.object({
  machine: z
    .string()
    .refine((v) => parseInt(v) < 0xffff, { error: "Value too big" }),
  serial: z
    .string()
    .refine((v) => parseInt(v) < 0xffff, { error: "Value too big" }),
  slaveId: z
    .string()
    .refine((v) => parseInt(v) > 0 && parseInt(v) < 0xff, {
      error: "Must be between 1 and 254",
    }),
});

type FormSchema = z.infer<typeof formSchema>;

const modbusMachines = machineProperties.filter((m) => m.modbus_rtu);

export function isModbusDeviceAssigned(device: ModbusDevice): boolean {
  return device.assignment != null;
}

export function ModbusAssignDialog({ device }: Props) {
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
      <ModbusAssignDialogContent device={device} key={key} setOpen={onClose} />
    </Dialog>
  );
}

type ContentProps = {
  device: ModbusDevice;
  setOpen: () => void;
};

export function ModbusAssignDialogContent({ device, setOpen }: ContentProps) {
  const client = useClient();
  const [isApplying, setIsApplying] = useState(false);
  const [isUnassigning, setUnassigning] = useState(false);
  const [writeSuccess, setWriteSuccess] = useState(false);

  const [numpadOpen, setNumpadOpen] = useState(false);
  const serialInputRef = useRef<HTMLInputElement | null>(null);

  const initialMachine = useMemo(
    () =>
      device.assignment?.machine_identification_unique.machine_identification.machine.toString(),
    [device],
  );

  const form = useForm<FormSchema>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      machine: initialMachine ?? "",
      serial: device.assignment?.machine_identification_unique.serial.toString() ?? "",
      slaveId: device.assignment?.slave_id.toString() ?? "1",
    },
    mode: "all",
  });
  const values = useFormValues(form);

  const isChangingMachine =
    initialMachine != null &&
    !!values.machine &&
    values.machine !== initialMachine;

  const isAssigned = isModbusDeviceAssigned(device);

  const performWrite = (values: FormSchema) =>
    client.writeModbusDeviceAssignment({
      port: device.port,
      device_machine_identification: {
        machine_identification_unique: {
          machine_identification: {
            vendor: VENDOR_QITECH,
            machine: parseInt(values.machine!),
          },
          serial: parseInt(values.serial!),
        },
        slave_id: parseInt(values.slaveId!),
      },
    });

  const performUnassign = () =>
    client.writeModbusDeviceAssignment({
      port: device.port,
      device_machine_identification: null,
    });

  const handleUnassign = () => {
    if (
      !window.confirm(
        "Unassigning removes this port from its machine. A backend restart is required and the machine will not work until reassigned. Continue?",
      )
    )
      return;
    setUnassigning(true);
    performUnassign()
      .then((res) => {
        if (res.success) {
          setWriteSuccess(true);
          form.reset({ machine: "", serial: "", slaveId: "1" });
          toast(
            <Toast title="Unassigned" icon="lu:CircleCheck">
              Assignment cleared. Restart required to apply changes.
            </Toast>,
          );
        }
      })
      .finally(() => setUnassigning(false));
  };

  const confirmIfChangingMachine = (): boolean => {
    if (!isChangingMachine) return true;
    return window.confirm(
      "Changing this port to another machine will disconnect it from the current setup. A backend restart is required. Continue?",
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

  useEffect(() => {
    if (numpadOpen && serialInputRef.current) {
      setTimeout(() => {
        if (serialInputRef.current) {
          serialInputRef.current.focus();
        }
      }, 0);
    }
  }, [numpadOpen]);

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

    const getCurrentValue = () => form.getValues("serial") || "";

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
      addDecimal: () => {},
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
          newValue = currentValue.slice(0, start) + currentValue.slice(end);
          newPosition = start;
        } else if (start > 0) {
          newValue = currentValue.slice(0, start - 1) + currentValue.slice(start);
          newPosition = start - 1;
        } else {
          return;
        }
        form.setValue("serial", newValue, { shouldValidate: true });
        updateCursorPosition(newPosition);
      },
      toggleSign: () => {},
      moveCursorLeft: () => {
        if (!serialInputRef.current) return;
        ensureFocus();
        const currentPos = serialInputRef.current.selectionStart || 0;
        if (currentPos > 0) {
          serialInputRef.current.setSelectionRange(currentPos - 1, currentPos - 1);
        }
      },
      moveCursorRight: () => {
        if (!serialInputRef.current) return;
        ensureFocus();
        const currentPos = serialInputRef.current.selectionStart || 0;
        const currentValue = getCurrentValue();
        if (currentPos < currentValue.length) {
          serialInputRef.current.setSelectionRange(currentPos + 1, currentPos + 1);
        }
      },
    };
  }, [form]);

  return (
    <DialogContent
      className="sm:max-w-2xl"
      onInteractOutside={(e) => e.preventDefault()}
      onPointerDownOutside={(e) => e.preventDefault()}
      onEscapeKeyDown={(e) => e.preventDefault()}
    >
      <DialogHeader>
        <DialogTitle className="text-xl">Machine Assignment</DialogTitle>
        <p className="text-base">for {device.port}</p>
        <DialogDescription className="text-base">
          To assign the port to a machine, select the machine, serial number &
          Modbus slave id.
        </DialogDescription>
      </DialogHeader>
      <Separator />
      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
          <FormField
            control={form.control}
            name="machine"
            render={({ field }) => (
              <FormItem>
                <FormLabel className="text-base">Maschine</FormLabel>
                <FormControl>
                  <Select {...field} onValueChange={field.onChange}>
                    <SelectTrigger className="h-12 min-w-48 text-base">
                      <SelectValue placeholder="Machine" />
                    </SelectTrigger>
                    <SelectContent>
                      {modbusMachines.map((machine) => (
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
          <FormField
            control={form.control}
            name="serial"
            render={({ field }) => (
              <FormItem>
                <FormLabel className="text-base">Serial</FormLabel>
                <FormControl>
                  <div className="flex flex-col gap-3">
                    <div className="flex items-center gap-2">
                      <Input
                        {...field}
                        ref={(element) => {
                          field.ref(element);
                          serialInputRef.current = element;
                        }}
                        placeholder="1234"
                        inputMode="numeric"
                        onFocus={() => setNumpadOpen(true)}
                        className="h-12 text-lg"
                      />
                      <Button
                        type="button"
                        variant="outline"
                        className="h-12 px-4 text-base"
                        onClick={() => setNumpadOpen((o) => !o)}
                      >
                        <Icon name={numpadOpen ? "lu:TouchpadOff" : "lu:Touchpad"} />
                        {numpadOpen ? "Hide" : "Numpad"}
                      </Button>
                    </div>
                    {numpadOpen && (
                      <div className="border-border bg-card flex justify-center rounded-xl border p-3 shadow-sm">
                        <TouchNumpad
                          onDigit={numpadHandlers.appendDigit}
                          onDelete={numpadHandlers.deleteChar}
                          onCursorLeft={numpadHandlers.moveCursorLeft}
                          onCursorRight={numpadHandlers.moveCursorRight}
                        />
                      </div>
                    )}
                  </div>
                </FormControl>
                <FormDescription>Serial number of the machine.</FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="slaveId"
            render={({ field }) => (
              <FormItem>
                <FormLabel className="text-base">Modbus Slave ID</FormLabel>
                <FormControl>
                  <Input
                    {...field}
                    placeholder="1"
                    inputMode="numeric"
                    className="h-12 max-w-32 text-lg"
                  />
                </FormControl>
                <FormDescription>
                  Modbus unit/slave address of the device.
                </FormDescription>
                <FormMessage />
              </FormItem>
            )}
          />
          <Separator />
          {isChangingMachine && (
            <Alert title="Changing machine assignment" variant="warning">
              This will disconnect the port from its current machine. Restart
              required to apply.
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
              disabled={!form.formState.isValid || isApplying || isUnassigning}
              onClick={() => setWriteSuccess(false)}
            >
              <Icon name="lu:Save" /> Save
            </Button>
            <Button
              type="button"
              variant="outline"
              disabled={!form.formState.isValid || isApplying || isUnassigning}
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
            <Button
              type="button"
              variant="destructive"
              disabled={!isAssigned || isApplying || isUnassigning}
              onClick={handleUnassign}
              aria-busy={isUnassigning}
              title={
                isAssigned
                  ? "Clears the machine assignment for this port. Restart is required for changes to take effect."
                  : "This port is not assigned to a machine."
              }
            >
              {isUnassigning ? (
                <>
                  <LoadingSpinner />
                  Unassigning…
                </>
              ) : (
                <>
                  <Icon name="lu:Unlink" />
                  Unassign
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
  );
}
