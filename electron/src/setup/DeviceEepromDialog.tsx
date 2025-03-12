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
import { useClient } from "@/hooks/useClient";
import { EthercatSetupEventDevice } from "@/hooks/useSocketio";
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
import { useFormValues } from "@/hooks/useFormValues";
import { validateU16 } from "@/lib/validation";
import { DeviceRoleComponent } from "@/components/DeviceRole";
import { Alert } from "@/components/Alert";
import { Separator } from "@/components/ui/separator";
import { Icon } from "@/components/Icon";
import { toast } from "sonner";
import { Toast } from "@/components/Toast";

type Props = {
  device: EthercatSetupEventDevice;
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
          Bearbeiten
        </Button>
      </DialogTrigger>
      <DeviceEeepromDialogContent device={device} key={key} setOpen={setOpen} />
    </Dialog>
  );
}

type ContentProps = {
  device: EthercatSetupEventDevice;
  setOpen: (open: boolean) => void;
};

export function DeviceEeepromDialogContent({ device, setOpen }: ContentProps) {
  const client = useClient();

  const form = useForm<FormSchema>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      machine:
        device.machine_device_identification?.machine_identification.machine.toString(),
      serial:
        device.machine_device_identification?.machine_identification.serial.toString(),
      role: device.machine_device_identification?.role.toString(),
    },
    mode: "all",
  });
  const values = useFormValues(form);
  useEffect(() => {
    console.log("values", values);
  }, [values]);

  const onSubmit = (values: FormSchema) => {
    client
      .writeMachineDeviceIdentification({
        subdevice_index: device.subdevice_index,
        machine_identification: {
          vendor: VENDOR_QITECH,
          serial: parseInt(values.serial!),
          machine: parseInt(values.machine!),
        },
        role: parseInt(values.role!),
      })
      .then((res) => {
        toast(
          <Toast title={"Gespeichert"} icon="lu:CircleCheck">
            Maschinenzuweisung erfolgreich gespeichert.
          </Toast>,
        );
        if (res.success) {
          setOpen(false);
        }
      });
  };

  const machinePreset = useMemo(() => {
    if (!values.machine) return;
    return getMachinePreset({
      vendor: VENDOR_QITECH,
      serial: 0,
      machine: parseInt(values.machine),
    });
  }, [values.machine]);

  const filteredAllowedDevices = useMemo(
    () =>
      filterAllowedDevices(
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
        <DialogTitle>Maschinenzuweisung</DialogTitle>
        <p>
          für {device.name}
          <Hex value={device.configured_address} />
        </p>
        <DialogDescription>
          Um das Gerät einer Maschine zuzuweisen, wählen Sie die Maschine,
          Seriennummer & Geräterolle aus.
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
                      <SelectValue placeholder="Maschine" />
                    </SelectTrigger>
                    <SelectContent>
                      {machinePresets.map((machine) => (
                        <SelectItem
                          key={machine.machine_id}
                          value={machine.machine_id.toString()}
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
                <FormLabel>Seriennummer</FormLabel>
                <FormControl>
                  <Input placeholder="1234" {...field} />
                </FormControl>
                <FormDescription>Seriennummer der Maschine.</FormDescription>
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
                <FormLabel>Rolle</FormLabel>
                <FormControl>
                  <Select {...field}>
                    <SelectTrigger className="w-min">
                      <SelectValue placeholder="Rolle" />
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
            <Icon name="lu:Save" /> Schreiben
          </Button>
          <Alert title="Neustart erforderlich" variant="info">
            Damit die Aenderungen wirksam werden, muss das Gerät neu gestartet
            werden
          </Alert>
        </form>
      </Form>
    </DialogContent>
  );
}
