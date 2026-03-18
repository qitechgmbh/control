import exposeContexts from "./ipc/context-exposer";

console.log("preload.js loaded");
exposeContexts();
