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
  getMachinePreset,
  machinePresets,
  VENDOR_QITECH,
} from "@/machines/types";
import React, { useEffect, useMemo } from "react";
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

/*



*/

type ContentProps = {
  device: Device;
  setOpen: (open: boolean) => void;
};

export function DeviceEeepromDialogContent({ device, setOpen }: ContentProps) {
  const client = useClient();

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

  console.log(device);
  console.log(device["device_identification"]["Ethercat"]);

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
    return getMachinePreset({
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
                      {machinePresets.map((machine) => (
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
                  <Input placeholder="1234" {...field} />
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
