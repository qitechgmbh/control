import { useEffect } from "react";

/**
 * This hook sets up global event listeners to ensure that whenever
 * an input field receives focus, the system keyboard is triggered.
 */
export function useSystemKeyboard() {
  useEffect(() => {
    // Function to ensure keyboard shows when input is focused
    const handleInputFocus = (event: FocusEvent) => {
      const target = event.target as HTMLElement;
      
      // Check if the focused element is an input, textarea, or contenteditable
      if (
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.getAttribute("contenteditable") === "true")
      ) {
        // Ensure the input has inputMode attribute if it's an input element
        if (target.tagName === "INPUT") {
          const input = target as HTMLInputElement;
          
          // If inputMode is not set, set it based on type
          if (!input.inputMode) {
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
          }
        }
        
        // Trigger a click event to ensure the keyboard appears
        setTimeout(() => {
          target.click();
          target.focus();
        }, 10);
      }
    };

    // Add event listener
    document.addEventListener("focusin", handleInputFocus, true);

    // Cleanup
    return () => {
      document.removeEventListener("focusin", handleInputFocus, true);
    };
  }, []);
}

