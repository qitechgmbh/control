import { useEffect } from "react";

/**
 * Hook to ensure the system keyboard appears when input fields are focused.
 * 
 * This hook:
 * - Sets inputMode attributes on all input fields (tells the system which keyboard type to show)
 * - Explicitly triggers the system on-screen keyboard via IPC when inputs are focused
 * 
 * The inputMode attribute is a standard HTML attribute that works across platforms.
 * The IPC call ensures the keyboard is explicitly shown if it doesn't appear automatically.
 */
export function useSystemKeyboard() {
  useEffect(() => {
    // Function to set inputMode based on input type
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
            input.inputMode = "email";
            break;
          case "url":
            input.inputMode = "url";
            break;
          case "search":
            input.inputMode = "search";
            break;
          default:
            // Only set to "text" if not already explicitly set
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

    // Function to show keyboard via IPC
    // This explicitly triggers the system keyboard if it doesn't appear automatically
    const showKeyboard = async () => {
      try {
        if (window.keyboard?.show) {
          await window.keyboard.show();
        }
      } catch (error) {
        // Silently fail if keyboard API is not available
        // The inputMode attribute should still work
      }
    };

    // Function to handle input focus
    const handleInputFocus = async (event: FocusEvent) => {
      const target = event.target as HTMLElement;
      
      if (
        target &&
        (target.tagName === "INPUT" || target.tagName === "TEXTAREA")
      ) {
        const input = target as HTMLInputElement | HTMLTextAreaElement;
        setInputMode(input);
        
        // Ensure the input has proper accessibility attributes
        // This helps the system detect the input field
        if (!target.getAttribute("role")) {
          target.setAttribute("role", "textbox");
        }
        if (!target.getAttribute("aria-label") && target.getAttribute("placeholder")) {
          target.setAttribute("aria-label", target.getAttribute("placeholder") || "");
        }
        
        // Explicitly trigger the system keyboard
        // This ensures the keyboard appears even if inputMode alone doesn't trigger it
        await showKeyboard();
      }
    };

    // Set inputMode for all existing inputs on mount
    const allInputs = document.querySelectorAll("input, textarea");
    allInputs.forEach((input) => {
      if (input instanceof HTMLInputElement || input instanceof HTMLTextAreaElement) {
        setInputMode(input);
      }
    });

    // Listen for input focus events
    document.addEventListener("focusin", handleInputFocus, true);

    // Cleanup
    return () => {
      document.removeEventListener("focusin", handleInputFocus, true);
    };
  }, []);
}

