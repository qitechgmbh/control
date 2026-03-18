import { getBridge } from "@ui/bridge";

export async function minimizeWindow() {
  await getBridge().window.minimize();
}
export async function maximizeWindow() {
  await getBridge().window.maximize();
}
export async function closeWindow() {
  await getBridge().window.close();
}
