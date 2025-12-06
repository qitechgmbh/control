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
    // Notify main process that virtual keyboard is now visible
    if (window.keyboard?.setVirtualKeyboardVisibility) {
      window.keyboard.setVirtualKeyboardVisibility(true);
    }
  }, []);

  const hideKeyboard = useCallback(() => {
    setIsVisible(false);
    activeInputRef.current = null;
    // Notify main process that virtual keyboard is now hidden
    if (window.keyboard?.setVirtualKeyboardVisibility) {
      window.keyboard.setVirtualKeyboardVisibility(false);
    }
  }, []);

  // Prevent keyboard from closing when window loses focus (e.g., clicking outside window)
  useEffect(() => {
    function handleWindowBlur() {
      // Don't close keyboard when window loses focus
      // The keyboard should only close when clicking inside the window
    }

    function handleWindowFocus() {
      // When window regains focus, ensure input still has focus if keyboard was visible
      if (isVisible && activeInputRef.current) {
        // Try to restore focus to the input
        activeInputRef.current.focus();
      }
    }

    window.addEventListener("blur", handleWindowBlur);
    window.addEventListener("focus", handleWindowFocus);
    return () => {
      window.removeEventListener("blur", handleWindowBlur);
      window.removeEventListener("focus", handleWindowFocus);
    };
  }, [isVisible]);

  // Global pointerdown handler for click-outside detection
  useEffect(() => {
    function handlePointerDown(e: PointerEvent) {
      const el = e.target as HTMLElement | null;

      // Only handle events within the current document
      // Ignore events from outside the window (e.g., clicks on other windows)
      if (!el || el.ownerDocument !== document) {
        return;
      }

      // If click is on any input/textarea → keep keyboard open (or let it open via focusin)
      // This must be checked FIRST, even if keyboard is not yet visible
      // This prevents closing the keyboard when clicking on an input field
      if (el.tagName === "INPUT" || el.tagName === "TEXTAREA") {
        return;
      }

      // If click is on a label that's associated with an input → keep keyboard open
      // Labels can trigger focus on inputs, so we should let that happen
      if (el.tagName === "LABEL") {
        const labelFor = el.getAttribute("for");
        if (labelFor) {
          const associatedInput = document.getElementById(labelFor);
          if (associatedInput && (associatedInput.tagName === "INPUT" || associatedInput.tagName === "TEXTAREA")) {
            return;
          }
        }
        // Also check if label contains an input
        if (el.querySelector("input, textarea")) {
          return;
        }
      }

      // If keyboard is not open, do nothing
      if (!isVisible) return;

      // If click is on the active input itself or inside it → keep keyboard open
      if (activeInputRef.current) {
        if (el === activeInputRef.current || activeInputRef.current.contains(el)) {
          return;
        }
      }

      // If click is in the keyboard → keep keyboard open
      // Use closest() to work with portals/dialogs
      if (el.closest('[data-virtual-keyboard-root="true"]')) {
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

  // Numpad handlers for numeric inputs
  const handleNumpadDigit = useCallback((digit: string) => {
    handleKeyPress(digit);
  }, [handleKeyPress]);

  const handleNumpadDecimal = useCallback(() => {
    const inputElement = activeInputRef.current;
    if (!inputElement) return;

    const start = inputElement.selectionStart ?? inputElement.value.length;
    const end = inputElement.selectionEnd ?? inputElement.value.length;
    const currentValue = inputElement.value;

    if (!currentValue.includes(".")) {
      // Add decimal at cursor position
      inputElement.value =
        currentValue.slice(0, start) + "." + currentValue.slice(end);
      inputElement.selectionStart = inputElement.selectionEnd = start + 1;
    } else {
      // Move existing decimal to cursor position
      const currentDecimalIndex = currentValue.indexOf(".");
      const valueWithoutDecimal = currentValue.replace(".", "");
      const adjustedStart = start > currentDecimalIndex ? start - 1 : start;
      inputElement.value =
        valueWithoutDecimal.slice(0, adjustedStart) +
        "." +
        valueWithoutDecimal.slice(adjustedStart);
      inputElement.selectionStart = inputElement.selectionEnd =
        adjustedStart + 1;
    }

    // Dispatch events
    inputElement.dispatchEvent(new Event("input", { bubbles: true }));
    inputElement.dispatchEvent(new Event("change", { bubbles: true }));
    inputElement.focus();
  }, []);

  const handleNumpadDelete = useCallback(() => {
    handleKeyPress("BACKSPACE");
  }, [handleKeyPress]);

  const handleNumpadToggleSign = useCallback(() => {
    const inputElement = activeInputRef.current;
    if (!inputElement) return;

    const start = inputElement.selectionStart ?? inputElement.value.length;
    const currentValue = inputElement.value;

    let newValue: string;
    let newPosition: number;

    if (currentValue === "" || currentValue === "0") {
      newValue = "-";
      newPosition = 1;
    } else if (currentValue.startsWith("-")) {
      newValue = currentValue.slice(1);
      newPosition = Math.max(0, start - 1);
    } else {
      newValue = "-" + currentValue;
      newPosition = start + 1;
    }

    inputElement.value = newValue;
    inputElement.selectionStart = inputElement.selectionEnd = newPosition;

    // Dispatch events
    inputElement.dispatchEvent(new Event("input", { bubbles: true }));
    inputElement.dispatchEvent(new Event("change", { bubbles: true }));
    inputElement.focus();
  }, []);

  const handleNumpadCursorLeft = useCallback(() => {
    const inputElement = activeInputRef.current;
    if (!inputElement) return;

    const start = inputElement.selectionStart ?? inputElement.value.length;
    if (start > 0) {
      inputElement.setSelectionRange(start - 1, start - 1);
    }
    inputElement.focus();
  }, []);

  const handleNumpadCursorRight = useCallback(() => {
    const inputElement = activeInputRef.current;
    if (!inputElement) return;

    const start = inputElement.selectionStart ?? inputElement.value.length;
    const valueLength = inputElement.value.length;
    if (start < valueLength) {
      inputElement.setSelectionRange(start + 1, start + 1);
    }
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
            onClose={hideKeyboard}
            inputType={inputType}
            onNumpadDigit={handleNumpadDigit}
            onNumpadDecimal={handleNumpadDecimal}
            onNumpadDelete={handleNumpadDelete}
            onNumpadToggleSign={handleNumpadToggleSign}
            onNumpadCursorLeft={handleNumpadCursorLeft}
            onNumpadCursorRight={handleNumpadCursorRight}
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

