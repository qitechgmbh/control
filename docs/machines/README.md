# Machine Documentation

This directory contains documentation for QiTech machines.

## Structure

```
docs/machines/
├── manuals/          # Machine operation manuals (markdown files)
│   ├── extruder.md
│   ├── winder.md
│   ├── laser.md
│   └── mock.md
├── extruder/         # Machine-specific documentation
├── winder/
├── laser/
└── mock/
```

## Images

All manual images are stored in: **`electron/public/images/manuals/[machine-name]/`**

For example:
- Winder images: `electron/public/images/manuals/winder/`
- Extruder images: `electron/public/images/manuals/extruder/`
- Laser images: `electron/public/images/manuals/laser/`

### Image References in Manuals

When referencing images in manual markdown files (located in `docs/machines/manuals/`), use relative paths:

```markdown
![Image description](../../../../electron/public/images/manuals/[machine-name]/image.png)
```

## Adding Images

1. Place images in: `electron/public/images/manuals/[machine-name]/`
2. Reference in markdown: `../../../../electron/public/images/manuals/[machine-name]/your-image.png`