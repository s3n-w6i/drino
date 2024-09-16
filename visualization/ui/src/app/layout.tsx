import * as React from "react";

import type { Metadata } from "next";
import "./globals.css";
import {TooltipProvider} from "@/components/ui/tooltip";

export const metadata: Metadata = {
  title: "drino Dashboard",
  description: "Inspect datasets from drino",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="bg-muted">
          <TooltipProvider>
            {children}
          </TooltipProvider>
      </body>
    </html>
  );
}
