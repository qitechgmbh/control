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

      // Store reference to DOM element to avoid React Compiler warnings
      // We're mutating DOM properties, not React state
      const inputElement = activeInput;
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

      // Restore focus to input after key press
      // This ensures the keyboard stays open when clicking buttons
      setTimeout(() => {
        if (inputElement && document.activeElement !== inputElement) {
          inputElement.focus();
          // Restore cursor position
          if (inputElement.selectionStart !== null) {
            inputElement.setSelectionRange(
              inputElement.selectionStart,
              inputElement.selectionEnd,
            );
          }
        }
      }, 10);
    },
    [activeInput],
  );

  const inputType =
    activeInput instanceof HTMLInputElement
      ? (activeInput.type as "text" | "number" | "email" | "tel")
      : "text";

  // Keep focus on the active input when clicking on keyboard
  const handleKeyboardClick = useCallback(() => {
    if (activeInput) {
      // Store reference to DOM element to avoid React Compiler warnings
      const inputElement = activeInput;
      // Restore focus to the input after a short delay
      // This prevents the keyboard from closing when clicking buttons
      setTimeout(() => {
        inputElement.focus();
        // Restore cursor position if it was set
        if (inputElement.selectionStart !== null) {
          inputElement.setSelectionRange(
            inputElement.selectionStart,
            inputElement.selectionEnd,
          );
        }
      }, 10);
    }
  }, [activeInput]);

  return (
    <VirtualKeyboardContext.Provider
      value={{ showKeyboard, hideKeyboard, isVisible }}
    >
      {children}
      {isVisible && activeInput && (
        <div onClick={handleKeyboardClick} onMouseDown={handleKeyboardClick}>
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

