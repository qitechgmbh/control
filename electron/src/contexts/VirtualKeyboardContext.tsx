import React, { createContext, useContext, useState, useCallback } from "react";
import { VirtualKeyboard } from "@/components/touch/VirtualKeyboard";

type InputElement = HTMLInputElement | HTMLTextAreaElement;

interface VirtualKeyboardContextValue {
  showKeyboard: (input: InputElement) => void;
  hideKeyboard: () => void;
  isVisible: boolean;
}

const VirtualKeyboardContext = createContext<
  VirtualKeyboardContextValue | undefined
>(undefined);

export function VirtualKeyboardProvider({
  children,
}: {
  children: React.ReactNode;
}) {
  const [activeInput, setActiveInput] = useState<InputElement | null>(null);
  const [isVisible, setIsVisible] = useState(false);

  const showKeyboard = useCallback((input: InputElement) => {
    setActiveInput(input);
    setIsVisible(true);
  }, []);

  const hideKeyboard = useCallback(() => {
    setIsVisible(false);
    // Don't clear activeInput immediately to allow for blur delay
    setTimeout(() => {
      setActiveInput(null);
    }, 200);
  }, []);

  const handleKeyPress = useCallback(
    (key: string) => {
      if (!activeInput) return;

      const start = activeInput.selectionStart ?? activeInput.value.length;
      const end = activeInput.selectionEnd ?? activeInput.value.length;

      if (key === "BACKSPACE") {
        if (start === end && start > 0) {
          activeInput.value =
            activeInput.value.slice(0, start - 1) +
            activeInput.value.slice(end);
          activeInput.selectionStart = activeInput.selectionEnd = start - 1;
        } else {
          activeInput.value =
            activeInput.value.slice(0, start) + activeInput.value.slice(end);
          activeInput.selectionStart = activeInput.selectionEnd = start;
        }
      } else {
        activeInput.value =
          activeInput.value.slice(0, start) +
          key +
          activeInput.value.slice(end);
        activeInput.selectionStart = activeInput.selectionEnd =
          start + key.length;
      }

      // Dispatch input event to trigger React state updates
      const event = new Event("input", { bubbles: true });
      activeInput.dispatchEvent(event);

      // Dispatch change event for form libraries
      const changeEvent = new Event("change", { bubbles: true });
      activeInput.dispatchEvent(changeEvent);
    },
    [activeInput],
  );

  const inputType =
    activeInput instanceof HTMLInputElement
      ? (activeInput.type as "text" | "number" | "email" | "tel")
      : "text";

  return (
    <VirtualKeyboardContext.Provider
      value={{ showKeyboard, hideKeyboard, isVisible }}
    >
      {children}
      {isVisible && activeInput && (
        <VirtualKeyboard
          onKeyPress={handleKeyPress}
          onClose={hideKeyboard}
          inputType={inputType}
        />
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

