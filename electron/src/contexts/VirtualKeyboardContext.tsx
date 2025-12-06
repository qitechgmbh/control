import React, {
  createContext,
  useContext,
  useState,
  useCallback,
  useRef,
} from "react";
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
  // Use ref for DOM element to avoid React Compiler warnings about mutating state
  const activeInputRef = useRef<InputElement | null>(null);
  const [isVisible, setIsVisible] = useState(false);

  const showKeyboard = useCallback((input: InputElement) => {
    activeInputRef.current = input;
    setIsVisible(true);
  }, []);

  const hideKeyboard = useCallback(() => {
    setIsVisible(false);
    // Don't clear activeInputRef immediately to allow for blur delay
    setTimeout(() => {
      activeInputRef.current = null;
    }, 200);
  }, []);

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

    // Restore focus to input after key press
    // This ensures the keyboard stays open when clicking buttons
    requestAnimationFrame(() => {
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
    });
  }, []);

  const inputType =
    activeInputRef.current instanceof HTMLInputElement
      ? (activeInputRef.current.type as "text" | "number" | "email" | "tel")
      : "text";

  // Keep focus on the active input when clicking on keyboard
  const handleKeyboardClick = useCallback((e: React.MouseEvent) => {
    const inputElement = activeInputRef.current;
    if (inputElement) {
      // Prevent the click from stealing focus
      e.preventDefault();
      e.stopPropagation();
      
      // Restore focus to the input immediately
      // Use requestAnimationFrame to ensure this happens after any blur events
      requestAnimationFrame(() => {
        inputElement.focus();
        // Restore cursor position if it was set
        if (inputElement.selectionStart !== null) {
          inputElement.setSelectionRange(
            inputElement.selectionStart,
            inputElement.selectionEnd,
          );
        }
      });
    }
  }, []);

  return (
    <VirtualKeyboardContext.Provider
      value={{ showKeyboard, hideKeyboard, isVisible }}
    >
      {children}
      {isVisible && activeInputRef.current && (
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

