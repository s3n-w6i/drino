import * as React from "react";

const Drino = React.forwardRef<
    HTMLDivElement,
    React.HTMLAttributes<HTMLDivElement>
>(({ className }) => {
  return (
      <svg
          className={className}
          xmlns="http://www.w3.org/2000/svg"
          width="24"
          height="24"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
      >
        <path d="M 11 17.5 C11 19 10.5 19 14.5 19"/>
        <path d="M 14.5 19 C14.5 16 16.5 15 21.5 15"/>
        <path d="M 15 11.5 C9 11.5 10.5 5 6 5"/>
        <path d="M 21.5 15 C19 12.5 18 11.5 15 11.5"/>
        <path d="M 6 5 C2.5 5 2.5 6.5 2.5 7.5"/>
        <path d="M 6 9.5 C4 9.5 2.5 9.5 2.5 7.5"/>
        <path d="M 6 9.5 C8.5 16.5 5 19 8.5 19"/>
      </svg>
  )
})

Drino.displayName = "Drino"

export default Drino