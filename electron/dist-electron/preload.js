"use strict";
const THEME_MODE_CURRENT_CHANNEL = "theme-mode:current";
const THEME_MODE_TOGGLE_CHANNEL = "theme-mode:toggle";
const THEME_MODE_DARK_CHANNEL = "theme-mode:dark";
const THEME_MODE_LIGHT_CHANNEL = "theme-mode:light";
const THEME_MODE_SYSTEM_CHANNEL = "theme-mode:system";
function exposeThemeContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("themeMode", {
    current: () => ipcRenderer.invoke(THEME_MODE_CURRENT_CHANNEL),
    toggle: () => ipcRenderer.invoke(THEME_MODE_TOGGLE_CHANNEL),
    dark: () => ipcRenderer.invoke(THEME_MODE_DARK_CHANNEL),
    light: () => ipcRenderer.invoke(THEME_MODE_LIGHT_CHANNEL),
    system: () => ipcRenderer.invoke(THEME_MODE_SYSTEM_CHANNEL)
  });
}
const WIN_MINIMIZE_CHANNEL = "window:minimize";
const WIN_MAXIMIZE_CHANNEL = "window:maximize";
const WIN_FULLSCREEN_CHANNEL = "window:fullscreen";
const WIN_CLOSE_CHANNEL = "window:close";
function exposeWindowContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("electronWindow", {
    minimize: () => ipcRenderer.invoke(WIN_MINIMIZE_CHANNEL),
    maximize: () => ipcRenderer.invoke(WIN_MAXIMIZE_CHANNEL),
    fullscreen: (value) => ipcRenderer.invoke(WIN_FULLSCREEN_CHANNEL, value),
    close: () => ipcRenderer.invoke(WIN_CLOSE_CHANNEL)
  });
}
const ENVIRONMENT_INFO = "environment:info";
function exposeEnvironmentContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("environment", {
    getInfo: () => ipcRenderer.invoke(ENVIRONMENT_INFO)
  });
}
const UPDATE_EXECUTE = "update:execute";
const UPDATE_LOG = "update:log";
const UPDATE_END = "update:end";
const UPDATE_CANCEL = "update:cancel";
const UPDATE_STEP = "update:step";
function exposeUpdateContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  let currentLogListener = null;
  let currentEndListener = null;
  let currentStepListener = null;
  contextBridge.exposeInMainWorld("update", {
    execute: (params) => ipcRenderer.invoke(UPDATE_EXECUTE, params),
    cancel: () => ipcRenderer.invoke(UPDATE_CANCEL),
    onLog: (callback) => {
      if (currentLogListener) {
        ipcRenderer.removeListener(UPDATE_LOG, currentLogListener);
      }
      currentLogListener = (_event, log) => {
        callback(log);
      };
      ipcRenderer.on(UPDATE_LOG, currentLogListener);
    },
    onEnd: (callback) => {
      if (currentEndListener) {
        ipcRenderer.removeListener(UPDATE_END, currentEndListener);
      }
      currentEndListener = (_event, params) => {
        callback(params);
      };
      ipcRenderer.on(UPDATE_END, currentEndListener);
    },
    onStep: (callback) => {
      if (currentStepListener) {
        ipcRenderer.removeListener(UPDATE_STEP, currentStepListener);
      }
      currentStepListener = (_event, params) => {
        callback(params);
      };
      ipcRenderer.on(UPDATE_STEP, currentStepListener);
    }
  });
}
const TROUBLESHOOT_REBOOT_HMI = "troubleshoot:reboot-hmi";
const TROUBLESHOOT_RESTART_BACKEND = "troubleshoot:restart-backend";
const TROUBLESHOOT_START_LOG_STREAM = "troubleshoot:start-log-stream";
const TROUBLESHOOT_STOP_LOG_STREAM = "troubleshoot:stop-log-stream";
const TROUBLESHOOT_LOG_DATA = "troubleshoot:log-data";
function exposeTroubleshootContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("troubleshoot", {
    rebootHmi: () => ipcRenderer.invoke(TROUBLESHOOT_REBOOT_HMI),
    restartBackend: () => ipcRenderer.invoke(TROUBLESHOOT_RESTART_BACKEND),
    startLogStream: () => ipcRenderer.invoke(TROUBLESHOOT_START_LOG_STREAM),
    stopLogStream: () => ipcRenderer.invoke(TROUBLESHOOT_STOP_LOG_STREAM),
    onLogData: (callback) => ipcRenderer.on(TROUBLESHOOT_LOG_DATA, (_event, log) => {
      callback(log);
    })
  });
}
const NIXOS_LIST_GENERATIONS = "nixos:list-generations";
const NIXOS_SET_GENERATION = "nixos:set-generation";
const NIXOS_DELETE_GENERATION = "nixos:delete-generation";
function exposeNixOSContext() {
  const { contextBridge, ipcRenderer } = window.require("electron");
  contextBridge.exposeInMainWorld("nixos", {
    listGenerations: () => ipcRenderer.invoke(NIXOS_LIST_GENERATIONS),
    setGeneration: (generationId) => ipcRenderer.invoke(NIXOS_SET_GENERATION, generationId),
    deleteGeneration: (generationId) => ipcRenderer.invoke(NIXOS_DELETE_GENERATION, generationId)
  });
}
function exposeContexts() {
  exposeWindowContext();
  exposeThemeContext();
  exposeEnvironmentContext();
  exposeUpdateContext();
  exposeTroubleshootContext();
  exposeNixOSContext();
}
console.log("preload.js loaded");
exposeContexts();
