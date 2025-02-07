import { Droplet, Flame, RotateCcw, Wrench } from "lucide-react";
import { SidbarItem, Sidebar } from "./components/Sidebar";
import { ConfigurationPage } from "./pages/Configuration";

function App() {
  return (
    <Sidebar
      initialActiveIndex={0}
      items={[
        {
          button: (props) => (
            <SidbarItem icon={<RotateCcw size={22} />} {...props}>
              Winder
            </SidbarItem>
          ),
          children: <> Hello 3</>,
        },
        {
          button: (props) => (
            <SidbarItem icon={<Flame size={22} />} {...props}>
              Extruder
            </SidbarItem>
          ),
          children: <>Hello 2</>,
        },
        {
          button: (props) => (
            <SidbarItem {...props} icon={<Droplet size={22} />} {...props}>
              Wasserkühlung
            </SidbarItem>
          ),
          children: <>Hello</>,
        },
        {
          button: (props) => (
            <SidbarItem
              {...props}
              icon={<Wrench size={22} />}
              parentLink="/configuration"
              link="/configuration/ethercat"
            >
              Konfiguration
            </SidbarItem>
          ),
          children: <ConfigurationPage />,
        },
      ]}
    />
  );
}

export default App;
