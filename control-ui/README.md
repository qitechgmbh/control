# Control UI

A TypeScript React component library for the QiTech Control system.

## Installation

```bash
npm install control-ui
```

## Usage

```typescript
import { Button } from 'control-ui';

function App() {
  return (
    <Button variant="primary" size="medium" onClick={() => console.log('Clicked!')}>
      Click me
    </Button>
  );
}
```

## Components

### Button

A customizable button component with multiple variants and sizes.

#### Props

- `variant`: 'primary' | 'secondary' | 'danger' (default: 'primary')
- `size`: 'small' | 'medium' | 'large' (default: 'medium')
- `disabled`: boolean (default: false)
- `onClick`: () => void
- `className`: string
- `children`: ReactNode

## Development

### Building

```bash
npm run build
```

### Testing

```bash
npm test
```

### Linting

```bash
npm run lint
```

### Formatting

```bash
npm run format
```

## License

MIT
