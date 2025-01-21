import { Droplet, EthernetPort, Factory, Flame, RotateCcw } from "lucide-react";
import {} from "./ui/collapsible";
import {
  Sidebar,
  SidebarContent,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "./ui/sidebar";
import Link from "next/link";

type Props = {
  x?: number;
};

export function AppSidebar({}: Props) {
  return (
    <Sidebar>
      <SidebarHeader className="p-2">
        <h1 className="text-2xl font-bold">QiTech Control 2</h1>
      </SidebarHeader>
      <SidebarContent className="p-2">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton>
              <Factory size={16} />
              Maschienen
            </SidebarMenuButton>
            <SidebarMenuSub>
              <SidebarMenuSubItem>
                <SidebarMenuSubButton>
                  <RotateCcw size={16} />
                  Winder
                </SidebarMenuSubButton>
              </SidebarMenuSubItem>
              <SidebarMenuSubItem>
                <SidebarMenuSubButton>
                  <Flame size={16} />
                  Extruder
                </SidebarMenuSubButton>
              </SidebarMenuSubItem>
              <SidebarMenuSubItem>
                <SidebarMenuSubButton>
                  <Droplet size={16} />
                  Wasserkülung
                </SidebarMenuSubButton>
              </SidebarMenuSubItem>
            </SidebarMenuSub>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton asChild>
              <Link href="/ethercat">
                <EthernetPort size={16} />
                EtherCAT
              </Link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarContent>
    </Sidebar>
  );
}
