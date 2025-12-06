import React, {
  createContext,
  useContext,
  useState,
  useCallback,
  useRef,
  useEffect,
} from "react";
import { VirtualKeyboard } from "@/components/touch/VirtualKeyboard";

type InputElement = HTMLInputElement | HTMLTextAreaElement;

interface VirtualKeyboardContextValue {
  showKeyboard: (input: InputElement) => void;
  hideKeyboard: () => void;
  isVisible: boolean;
  keyboardRootRef: React.RefObject<HTMLDivElement | null>;
  activeInputRef: React.RefObject<InputElement | null>;
}

const VirtualKeyboardContext = createContext<
  VirtualKeyboardContextValue | undefined
>(undefined);

export function VirtualKeyboardProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  // Use ref for DOM element to avoid React Compiler warnings about mutating state
  const activeInputRef = useRef<InputElement | null>(null);
  const keyboardRootRef = useRef<HTMLDivElement | null>(null);
  const [isVisible, setIsVisible] = useState(false);

  const showKeyboard = useCallback((input: InputElement) => {
    activeInputRef.current = input;
    input.focus();
    setIsVisible(true);
  }, []);

  const hideKeyboard = useCallback(() => {
    setIsVisible(false);
    activeInputRef.current = null;
  }, []);

  // Global pointerdown handler for click-outside detection
  useEffect(() => {
    function handlePointerDown(e: PointerEvent) {
      const el = e.target as HTMLElement | null;

      // If click is on any input/textarea → keep keyboard open (or let it open via focusin)
      // This must be checked FIRST, even if keyboard is not yet visible
      // This prevents closing the keyboard when clicking on an input field
      if (el && (el.tagName === "INPUT" || el.tagName === "TEXTAREA")) {
        return;
      }

      // If keyboard is not open, do nothing
      if (!isVisible) return;

      // If click is on the active input itself or inside it → keep keyboard open
      if (activeInputRef.current && el) {
        if (el === activeInputRef.current || activeInputRef.current.contains(el)) {
          return;
        }
      }

      // If click is in the keyboard → keep keyboard open
      // Use closest() to work with portals/dialogs
      if (el && el.closest('[data-virtual-keyboard-root="true"]')) {
        return;
      }

      // Click is outside both input and keyboard → close keyboard
      hideKeyboard();
    }

    document.addEventListener("pointerdown", handlePointerDown);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
    };
  }, [isVisible, hideKeyboard]);

  const handleKeyPress = useCallback((key: string) => {
    const inputElement = activeInputRef.current;
    if (!inputElement) return;

    const start = inputElement.selectionStart ?? inputElement.value.length;
    const end = inputElement.selectionEnd ?? inputElement.value.length;

    if (key === "BACKSPACE") {
      if (start === end && start > 0) {
        inputElement.value =
          inputElement.value.slice(0, start - 1) +
          inputElement.value.slice(end);
        inputElement.selectionStart = inputElement.selectionEnd = start - 1;
      } else {
        inputElement.value =
          inputElement.value.slice(0, start) + inputElement.value.slice(end);
        inputElement.selectionStart = inputElement.selectionEnd = start;
      }
    } else {
      inputElement.value =
        inputElement.value.slice(0, start) +
        key +
        inputElement.value.slice(end);
      inputElement.selectionStart = inputElement.selectionEnd =
        start + key.length;
    }

    // Dispatch input event to trigger React state updates
    const event = new Event("input", { bubbles: true });
    inputElement.dispatchEvent(event);

    // Dispatch change event for form libraries
    const changeEvent = new Event("change", { bubbles: true });
    inputElement.dispatchEvent(changeEvent);

    // Important: Keep focus on input after key press
    inputElement.focus();
  }, []);

  const inputType =
    activeInputRef.current instanceof HTMLInputElement
      ? (activeInputRef.current.type as "text" | "number" | "email" | "tel")
      : "text";

  return (
    <VirtualKeyboardContext.Provider
      value={{
        showKeyboard,
        hideKeyboard,
        isVisible,
        keyboardRootRef,
        activeInputRef,
      }}
    >
      {children}
      {isVisible && activeInputRef.current && (
        <div ref={keyboardRootRef}>
          <VirtualKeyboard
            onKeyPress={handleKeyPress}
            onClose={hideKeyboard}
            inputType={inputType}
          />
        </div>
      )}
    </VirtualKeyboardContext.Provider>
  );
}

export function useVirtualKeyboard() {
  const context = useContext(VirtualKeyboardContext);
  if (context === undefined) {
    throw new Error(
      "useVirtualKeyboard must be used within a VirtualKeyboardProvider",
    );
  }
  return context;
}

