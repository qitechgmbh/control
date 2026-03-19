import "@ui/styles/global.css";
import "@ui/styles/markdown.css";

import { setBridge } from "@ui/bridge";
import { tauriBridge } from "./bridge";

// Register the Tauri bridge before the app boots
setBridge(tauriBridge);

import "./App";
