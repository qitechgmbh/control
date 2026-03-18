import "@ui/styles/global.css";
import "@ui/styles/markdown.css";

import { setBridge } from "@ui/bridge";
import { electronBridge } from "@/bridge";

// Register the Electron bridge before the app boots
setBridge(electronBridge);

import "@/App";
