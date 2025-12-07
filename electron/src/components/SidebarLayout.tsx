import { OutsideCorner } from "@/components/OutsideCorner";
import { useMachines } from "@/client/useMachines";
import { useOnSubpath } from "@/lib/useOnSubpath";
import {
  Link,
  Outlet,
  useRouter,
  useRouterState,
} from "@tanstack/react-router";
import { Fragment, useEffect } from "react";
import React from "react";
import { Icon, IconName } from "./Icon";
import { useMainNamespace } from "@/client/mainNamespace";

type SidebarItemContent = {
  link: string;
  activeLink: string;
  icon?: IconName;
  title: string;
};

type SidebarItemProps = SidebarItemContent & {
  isFirst: boolean;
};

export function SidebarItem({
  link,
  icon,
  title,
  isFirst,
  activeLink,
}: SidebarItemProps) {
  const isActive = useOnSubpath(activeLink);
  return (
    <Link
      to={link}
      className={`relative h-18 w-full ${isActive ? "pl-2" : "px-2"}`}
    >
      <div
        className={`text-md relative z-10 flex h-full w-full items-center justify-center gap-2 ${
          isActive ? "rounded-l-lg bg-white pr-2" : "rounded-lg bg-neutral-100"
        }`}
      >
        {icon && <Icon name={icon} />}
        {title}
      </div>
      {isActive && <OutsideCorner rightTop={!isFirst} rightBottom={true} />}
    </Link>
  );
}

// Create a Width Context
const SidebarlessWidthContext = React.createContext<number | null>(null);

// Create a hook to use the width context
export function useSidebarlessWidth() {
  const width = React.useContext(SidebarlessWidthContext);
  if (width === null) {
    throw new Error("useWidth must be used within a WidthProvider");
  }
  return width;
}

export function SidebarLayout() {
  const machines = useMachines();
  const router = useRouter();
  const routerState = useRouterState();
  const { machines: machinesEvent } = useMainNamespace();
  const [contentWidth, setContentWidth] = React.useState<number>(0);

  // Machine connection guard: redirect to setup if selected machine disconnects
  useEffect(() => {
    // Wait for machines data to be loaded
    if (!machinesEvent?.data) return;

    // Extract serial from current path (e.g., /machines/laser1/12345/control)
    const pathMatch = routerState.location.pathname.match(
      /\/machines\/[^/]+\/(\d+)/,
    );
    if (!pathMatch) return; // Not on a machine page

    // Safety check: ensure pathMatch has at least 2 elements (full match + capture group)
    if (pathMatch.length < 2) {
      console.warn("[ConnectionGuard] Invalid pathMatch structure:", pathMatch);
      return;
    }

    // Parse serial number with error handling
    const serialString = pathMatch[1];
    const serialNumber = parseInt(serialString, 10);

    // Check if parseInt succeeded (NaN means parsing failed)
    if (isNaN(serialNumber)) {
      console.warn(
        `[ConnectionGuard] Failed to parse serial number from "${serialString}"`,
        { pathname: routerState.location.pathname },
      );
      return;
    }

    // Check if machine still exists in the raw machines data
    const machineExists = machinesEvent.data.machines.some(
      (m) => m.machine_identification_unique.serial === serialNumber,
    );

    if (!machineExists) {
      router.navigate({ to: "/_sidebar/setup/machines" });
    }
  }, [machines, machinesEvent, routerState.location.pathname, router]);

  const items: SidebarItemContent[] = [
    ...machines.map((machine) => ({
      link: `/_sidebar/machines/${machine.slug}/${machine.machine_identification_unique.serial}/control`,
      activeLink: `/_sidebar/machines/${machine.slug}/${machine.machine_identification_unique.serial}`,
      title: machine.name,
      icon: machine.icon,
    })),
    {
      link: "/_sidebar/setup/ethercat",
      activeLink: "/_sidebar/setup",
      title: "Setup",
      icon: "lu:Settings2",
    },
  ];

  // width measuring
  const outletRef = React.useRef<HTMLDivElement>(null);
  React.useEffect(() => {
    if (outletRef.current) {
      // Set initial width
      setContentWidth(outletRef.current.offsetWidth);

      // Create a ResizeObserver to track width changes
      const resizeObserver = new ResizeObserver((entries) => {
        for (const entry of entries) {
          setContentWidth(entry.contentRect.width);
        }
      });

      resizeObserver.observe(outletRef.current);

      // Clean up observer on unmount
      return () => {
        resizeObserver.disconnect();
      };
    }
  }, []);

  return (
    <SidebarlessWidthContext.Provider value={contentWidth}>
      <div className="fixed flex h-full w-48 flex-col bg-neutral-200">
        <div className="flex h-18 flex-col items-center justify-center gap-0 pt-2">
          <div className="font-qitech line-clamp-none text-3xl">QITECH</div>
        </div>
        <div className="flex flex-col gap-2">
          {items.map((item, index) => (
            <Fragment key={item.link}>
              <SidebarItem {...item} isFirst={index === 0} />
            </Fragment>
          ))}
        </div>
      </div>
      <div className="ml-48" ref={outletRef}>
        <Outlet />
      </div>
    </SidebarlessWidthContext.Provider>
  );
}
