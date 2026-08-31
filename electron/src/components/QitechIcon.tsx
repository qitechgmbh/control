import React from "react";

export const qitechIcons = {
  Extruder: Extruder,
};

function Extruder({ className }: { className?: string }) {
  return (
    <svg
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M2 17V15L6.97707 12.2495C7.27322 12.0859 7.60606 12 7.94444 12H20C21.1046 12 22 12.8954 22 14V18C22 19.1046 21.1046 20 20 20H7.94444C7.60606 20 7.27322 19.9141 6.97707 19.7505L2 17Z"
        stroke="black"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
      <path
        d="M14 12V9.16205C14 8.92811 13.918 8.70158 13.7682 8.52187L11.3668 5.64018C10.824 4.98886 11.2872 4 12.135 4H19.865C20.7128 4 21.176 4.98886 20.6332 5.64018L18.2318 8.52187C18.082 8.70158 18 8.92811 18 9.16205V12"
        stroke="black"
        strokeWidth="2"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
