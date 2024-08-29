import type { Metadata } from "next";
import { Inter } from "next/font/google";
import { Analytics } from "@vercel/analytics/react";
import { SpeedInsights } from "@vercel/speed-insights/next";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: "bulloak - Test Generator using Branching Tree Technique",
  description: "bulloak is a powerful test generator that implements the Branching Tree Technique (BTT) for comprehensive smart contract testing.",
  keywords: "bulloak, Solidity, test generator, Branching Tree Technique, BTT, smart contracts, Ethereum",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={`${inter.className}`}>{children}</body>
    </html>
  );
}
