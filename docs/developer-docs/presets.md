# Presets

To allow the user to save the current configuration of a machine, they can create a preset in the presets page. Presets are presited in electron's local storage.

Presets are all handled by the `PresetsPage`. To use the `PresetsPage` for a single machine, one must implement the following.

Create a zod schema for the data, the presets contain.
 ```tsx
const mock1PresetDataSchema = z
  .object({
    frequency1: z.number(),
    frequency2: z.number(),
    frequency3: z.number(),
  })
  .partial();

type Mock1 = typeof mock1PresetDataSchema;
```
Here, intensionally, `Mock1` is the schema type itself. It can be used as a type argument everything that uses presets.

Next, create a map containing all schema versions currently supported (the new schema, to be precise).
```tsx
const schemas = new Map([[1, mock1PresetDataSchema]]);
```
New, the schema will be used to parse version `1`. To create new versions, see migrations below.

One also needs to define a table, how the preset data should be shown to the user.
```tsx
const previewEntries: PresetPreviewEntries<Mock1> = [
  {
    name: "Frequency 1",
    unit: "mHz",
    renderValue: (data: PresetData<Mock1>) => data?.frequency1?.toFixed(3),
  },
  {
    name: "Frequency 2",
    unit: "mHz",
    renderValue: (data: PresetData<Mock1>) => data?.frequency2?.toFixed(3),
  },
  {
    name: "Frequency 3",
    unit: "mHz",
    renderValue: (data: PresetData<Mock1>) => data?.frequency3?.toFixed(3),
  },
];
```
`renderValue` gets the data itself and renders the corresponding part of it. Returning undefined will automatically be rendered as the value is not set, showing "N/A".

Create a function for the new react component and use the appropreate machine.
```tsx
export function Mock1PresetsPage() {
  const { setFrequency1, setFrequency2, setFrequency3, defaultState, state } =
    useMock1();
  // ...
}
```

Here, two more functions need to be created.     const toPresetData = (s: typeof state): PresetData<Mock1> => ({
    frequency1: s?.frequency1 ?? defaultState?.frequency1,
    frequency2: s?.frequency2 ?? defaultState?.frequency2,
    frequency3: s?.frequency3 ?? defaultState?.frequency3,The first takes in a preset and applies it on the machine. If data is missing, use the default defined by the backend.
```tsx
export function Mock1PresetsPage() {
  const { setFrequency1, setFrequency2, setFrequency3, defaultState, state } =
    useMock1();

  const applyPreset = (preset: Preset<Mock1>) => {
    const frequency1 = preset.data?.frequency1 ?? 100;
    const frequency2 = preset.data?.frequency2 ?? 200;
    const frequency3 = preset.data?.frequency3 ?? 500;

    setFrequency1(frequency1);
    setFrequency2(frequency2);
    setFrequency3(frequency3);
  };
  // ...
}
```
The later does the opposite, it turns a machine state, comming from the backend, into the preset data
```tsx
export function Mock1PresetsPage() {
  const { setFrequency1, setFrequency2, setFrequency3, defaultState, state } =
    useMock1();
  // ...
    const toPresetData = (s: typeof state): PresetData<Mock1> => ({
    frequency1: s?.frequency1 ?? defaultState?.frequency1,
    frequency2: s?.frequency2 ?? defaultState?.frequency2,
    frequency3: s?.frequency3 ?? defaultState?.frequency3,
  });
  // ...
}
```

Having all this, we can simple pass it to the `PresetsPage`
```tsx
export function Mock1PresetsPage() {
  // ...
  return (
    <PresetsPage
      machine_identification={mock1.machine_identification}
      currentState={toPresetData(state)}
      schemas={schemas}
      schemaVersion={1}
      applyPreset={applyPreset}
      previewEntries={previewEntries}
      defaultState={toPresetData(defaultState)}
    />
  );
}
```
Also mind the machine identification, that has to match the machine we are using and the schema versionen. Here we pass `1`, as we also used `1` in the schemas map.

## Migrations

Migrations are handled by the `PresetsPage`, however, one still has to define what to migrate.
To do so, first create the new schema.
```tsx
const fancyNewMock1PresetDataSchema = z
  .object({
    frequency1_now_with_ai: z.number(),
    frequency2: z.number(),
    frequency3: z.number(),
    frequency4: z.number(),
  })
  .partial();

type Mock1 = typeof fancyNewMock1PresetDataSchema;
```
and then, also create a migration that uses the new schema, but can handle the old format.
```tsx
const mock1PresetDataSchemaMigration: PresetSchema = z.preprocess(
  (obj: any): PresetData<Mock1> => ({
    frequency1_now_with_ai: obj?.frequency1, // Here, a field have been renamed
    frequency2: obj?.frequency2,
    frequency3: obj?.frequency3,
    frequency4: 666.0, // A new field is added. Initialize it with default the value 
  }),
  fancyNewMock1PresetDataSchema,
);
```
Usings zods preprocessing step,
we can first transform the data as needed and then validate using the new schema accordingly.

We have to register both schemas in our schema map.
```tsx
const schemas = new Map<number, Mock1>([
  [1, mock1PresetDataSchemaMigration as Mock1],
  [2, fancyNewMock1PresetDataSchema],
]);
```
Mind the cast, which is ok, because `mock1PresetDataSchemaMigration` parses `fancyNewMock1PresetDataSchema` by construction.

Lastly, simply change the schema version being passed to the `PresetsPage`.
```diff
@@ -62,7 +77,7 @@ export function Mock1PresetsPage() {
       machine_identification={mock1.machine_identification}
       currentState={toPresetData(state)}
       schemas={schemas}
-      schemaVersion={1}
+      schemaVersion={2}
       applyPreset={applyPreset}
       previewEntries={previewEntries}
       defaultState={toPresetData(defaultState)}
```
Make sure, that all (exsiting and new migrations) return a schema version 2 conform format.
Of corse, one can chain migrations of needed - just the end result must be complient.
