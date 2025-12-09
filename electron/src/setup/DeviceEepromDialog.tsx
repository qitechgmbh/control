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
import { validateU16 } from "@/lib/validation";
import { DeviceRoleComponent } from "@/components/DeviceRole";
import { Alert } from "@/components/Alert";
import { Separator } from "@/components/ui/separator";
import { Icon } from "@/components/Icon";
import { toast } from "sonner";
import { Toast } from "@/components/Toast";
import { EthercatDevicesEventData } from "@/client/mainNamespace";
import { TouchNumpad } from "@/components/touch/TouchNumpad";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

type Device = NonNullable<EthercatDevicesEventData["Done"]>["devices"][number];

type Props = {
  device: Device;
};

const formSchema = z.object({
  machine: z.string().superRefine(validateU16),
  serial: z.string().superRefine(validateU16),
  role: z.string().superRefine(validateU16),
});

type FormSchema = z.infer<typeof formSchema>;

export function DeviceEepromDialog({ device }: Props) {
  const [open, setOpen] = React.useState(false);
  const key = useMemo(() => Math.random(), [open]);
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="outline">
          <Icon name="lu:Pencil" />
          Assign
        </Button>
      </DialogTrigger>
      <DeviceEeepromDialogContent device={device} key={key} setOpen={setOpen} />
    </Dialog>
  );
}

// (Empty comment block removed)
type ContentProps = {
  device: Device;
  setOpen: (open: boolean) => void;
};

export function DeviceEeepromDialogContent({ device, setOpen }: ContentProps) {
  const client = useClient();
  const serialInputRef = useRef<HTMLInputElement>(null);
  const [numpadOpen, setNumpadOpen] = useState(false);

  const form = useForm<FormSchema>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      machine:
        device.device_identification.device_machine_identification?.machine_identification_unique.machine_identification.machine.toString(),
      serial:
        device.device_identification.device_machine_identification?.machine_identification_unique.serial.toString(),
      role: device.device_identification.device_machine_identification?.role.toString(),
    },
    mode: "all",
  });
  const values = useFormValues(form);

  // Removed unnecessary console.log statements.

  const onSubmit = (values: FormSchema) => {
    client
      .writeMachineDeviceIdentification({
        hardware_identification_ethercat: {
          subdevice_index:
            device.device_identification.device_hardware_identification
              .Ethercat!.subdevice_index,
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
      })
      .then((res) => {
        if (res.success) {
          toast(
            <Toast title={"Gespeichert"} icon="lu:CircleCheck">
              Machine assignment written successfully.
            </Toast>,
          );
          setOpen(false);
        }
      });
  };

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
    <DialogContent>
      <DialogHeader>
        <DialogTitle>Machine Assignment</DialogTitle>
        <p>
          for {device.name}
          <Hex value={device.configured_address} />
        </p>
        <DialogDescription>
          To assign the device to a machine, select the machine, serial number &
          device role.
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
                  <div className="flex items-center gap-2">
                    <Input
                      {...field}
                      ref={(e) => {
                        field.ref(e);
                        serialInputRef.current = e;
                      }}
                      placeholder="1234"
                      onFocus={() => setNumpadOpen(true)}
                    />
                    <Popover open={numpadOpen} onOpenChange={setNumpadOpen}>
                      <PopoverTrigger asChild>
                        <Button
                          type="button"
                          variant="outline"
                          size="icon"
                          aria-label="Toggle numpad"
                          onClick={(e) => {
                            e.preventDefault();
                            setNumpadOpen(!numpadOpen);
                          }}
                        >
                          <Icon name="lu:Calculator" />
                        </Button>
                      </PopoverTrigger>
                      <PopoverContent className="w-auto p-4" align="end">
                        <TouchNumpad
                          onDigit={numpadHandlers.appendDigit}
                          onDelete={numpadHandlers.deleteChar}
                          onCursorLeft={numpadHandlers.moveCursorLeft}
                          onCursorRight={numpadHandlers.moveCursorRight}
                        />
                      </PopoverContent>
                    </Popover>
                  </div>
                </FormControl>
                <FormDescription>Serial number of the machine.</FormDescription>
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
          <Button type="submit" disabled={!form.formState.isValid}>
            <Icon name="lu:Save" /> Write
          </Button>
          <Alert title="Restart mandatory" variant="info">
            The device must be restarted for the changes to take effect
          </Alert>
        </form>
      </Form>
    </DialogContent>
  );
}
