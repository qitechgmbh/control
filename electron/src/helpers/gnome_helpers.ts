export async function gnomeShowVirtualKeyboard() {
  await window.gnome.showVirtualKeyboard();
}

export async function gnomeHideVirtualKeyboard() {
  await window.gnome.hideVirtualKeyboard();
}
