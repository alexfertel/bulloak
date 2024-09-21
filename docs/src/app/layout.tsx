import type { Metadata } from "next";
import { Analytics } from "@vercel/analytics/react";
import { SpeedInsights } from "@vercel/speed-insights/next";
import { mono } from "./fonts";
import "./globals.css";

const title = "bulloak - Test Generator using Branching Tree Technique";
const description =
  "bulloak is a powerful test generator that implements the Branching Tree Technique (BTT) for comprehensive smart contract testing.";

export const metadata: Metadata = {
  metadataBase: new URL("https://www.bulloak.dev"),
  alternates: {
    canonical: "/",
  },
  title,
  description,
  openGraph: {
    title,
    description,
    images: `/opengraph-image.png`,
  },
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
