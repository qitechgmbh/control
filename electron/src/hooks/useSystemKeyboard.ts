import { useEffect } from "react";

/**
 * Hook to ensure inputMode attributes are set on all input fields.
 * 
 * This ensures that system keyboards (on-screen keyboards) appear when inputs are focused.
 * Works on all platforms and desktop environments (GNOME, KDE, Windows, macOS, mobile).
 * 
 * The inputMode attribute is a standard HTML attribute that tells the system
 * which type of keyboard to show. This works regardless of the desktop environment.
 */
export function useSystemKeyboard() {
  useEffect(() => {
    // Function to set inputMode based on input type
    const setInputMode = (input: HTMLInputElement | HTMLTextAreaElement) => {
      // Skip if already set or if it's a hidden input
      if (input instanceof HTMLInputElement) {
        if (input.inputMode || input.type === "hidden") {
          return;
        }
        
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
            input.inputMode = "text";
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
        setInputMode(target as HTMLInputElement | HTMLTextAreaElement);
      }
    };

    // Set inputMode for all existing inputs on mount
    const allInputs = document.querySelectorAll("input, textarea");
    allInputs.forEach((input) => {
      if (input instanceof HTMLInputElement || input instanceof HTMLTextAreaElement) {
        setInputMode(input);
      }
    });

    // Listen for new inputs being focused
    document.addEventListener("focusin", handleInputFocus, true);

    // Cleanup
    return () => {
      document.removeEventListener("focusin", handleInputFocus, true);
    };
  }, []);
}

