import { useEffect } from "react";
import { useVirtualKeyboard } from "@/contexts/VirtualKeyboardContext";

/**
 * Hook to ensure the virtual keyboard appears when input fields are focused.
 * 
 * This hook:
 * - Sets inputMode attributes on all input fields (for accessibility)
 * - Shows the virtual keyboard when inputs are focused
 * - Hides the virtual keyboard when inputs lose focus
 */
export function useSystemKeyboard() {
  const { showKeyboard, hideKeyboard } = useVirtualKeyboard();

  useEffect(() => {
    // Function to set inputMode based on input type
    // This is still useful for accessibility and system integration
    const setInputMode = (input: HTMLInputElement | HTMLTextAreaElement) => {
      // Skip hidden inputs
      if (input instanceof HTMLInputElement && input.type === "hidden") {
        return;
      }
      
      if (input instanceof HTMLInputElement) {
        // Set inputMode based on type
        switch (input.type) {
          case "number":
          case "tel":
            input.inputMode = "numeric";
            break;
          case "email":
          case "url":
          case "search":
            input.inputMode = input.type;
            break;
          default:
            if (!input.inputMode) {
              input.inputMode = "text";
            }
        }
      } else if (input instanceof HTMLTextAreaElement) {
        if (!input.inputMode) {
          input.inputMode = "text";
        }
      }
    };

    // Function to handle input focus
    const handleInputFocus = (event: FocusEvent) => {
      const target = event.target as HTMLElement;
      
      if (
        target &&
        (target.tagName === "INPUT" || target.tagName === "TEXTAREA")
      ) {
        const input = target as HTMLInputElement | HTMLTextAreaElement;
        setInputMode(input);
        
        // Ensure the input has proper accessibility attributes
        if (!target.getAttribute("role")) {
          target.setAttribute("role", "textbox");
        }
        if (!target.getAttribute("aria-label") && target.getAttribute("placeholder")) {
          target.setAttribute("aria-label", target.getAttribute("placeholder") || "");
        }
        
        // Show virtual keyboard
        showKeyboard(input);
      }
    };

    // Function to handle input blur
    // Use a small delay to allow for keyboard button clicks
    let blurTimeout: NodeJS.Timeout | null = null;
    const handleInputBlur = (event: FocusEvent) => {
      const target = event.target as HTMLElement;
      
      if (
        target &&
        (target.tagName === "INPUT" || target.tagName === "TEXTAREA")
      ) {
        // Clear any existing timeout
        if (blurTimeout) {
          clearTimeout(blurTimeout);
        }
        
        // Delay hiding keyboard to allow for button clicks
        blurTimeout = setTimeout(() => {
          hideKeyboard();
        }, 200);
      }
    };

    // Set inputMode for all existing inputs on mount
    const allInputs = document.querySelectorAll("input, textarea");
    allInputs.forEach((input) => {
      if (input instanceof HTMLInputElement || input instanceof HTMLTextAreaElement) {
        setInputMode(input);
      }
    });

    // Listen for input focus and blur events
    document.addEventListener("focusin", handleInputFocus, true);
    document.addEventListener("focusout", handleInputBlur, true);

    // Cleanup
    return () => {
      document.removeEventListener("focusin", handleInputFocus, true);
      document.removeEventListener("focusout", handleInputBlur, true);
      if (blurTimeout) {
        clearTimeout(blurTimeout);
      }
    };
  }, [showKeyboard, hideKeyboard]);
}
