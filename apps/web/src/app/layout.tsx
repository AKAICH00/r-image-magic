import type { Metadata } from "next";
import { ClerkProvider } from "@clerk/nextjs";
import { IBM_Plex_Mono, Manrope, Space_Grotesk } from "next/font/google";

import { isClerkConfigured } from "@/lib/env";
import "./globals.css";

const bodyFont = Manrope({
  variable: "--font-body",
  subsets: ["latin"],
});

const displayFont = Space_Grotesk({
  variable: "--font-display",
  subsets: ["latin"],
});

const monoFont = IBM_Plex_Mono({
  variable: "--font-mono-ui",
  subsets: ["latin"],
  weight: ["400", "500"],
});

export const metadata: Metadata = {
  metadataBase: new URL("https://meetmockup.com"),
  title: {
    default: "MeetMockup — Realistic Product Mockups",
    template: "%s | MeetMockup",
  },
  description:
    "Generate photorealistic product mockups with displacement mapping. API-first, built for POD sellers and developers.",
};

function AppDocument({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={`${bodyFont.variable} ${displayFont.variable} ${monoFont.variable} min-h-screen bg-background text-foreground antialiased`}
      >
        {children}
      </body>
    </html>
  );
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  if (!isClerkConfigured()) {
    return <AppDocument>{children}</AppDocument>;
  }

  return (
    <ClerkProvider>
      <AppDocument>{children}</AppDocument>
    </ClerkProvider>
  );
}
