import type { Metadata } from "next";
import { Analytics } from "@vercel/analytics/react";
import { SpeedInsights } from "@vercel/speed-insights/next";
import { mono } from "./fonts";
import "./globals.css";

export const metadata: Metadata = {
  title: "bulloak - Test Generator using Branching Tree Technique",
  description:
    "bulloak is a powerful test generator that implements the Branching Tree Technique (BTT) for comprehensive smart contract testing.",
  keywords:
    "bulloak, Solidity, test generator, Branching Tree Technique, BTT, smart contracts, Ethereum",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${mono.className} antialiased touch-manipulation`}>
        {children}

        <Analytics />
        <SpeedInsights />
      </body>
    </html>
  );
}
