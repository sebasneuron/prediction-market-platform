import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";

import { Provider } from "@/components/ui/provider";
import Navbar from "@/components/Navbar";

import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Polymarket clone",
  description: "Next level system designed for prediction markets",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <Provider defaultTheme="light">
          <Navbar />
          {children}
        </Provider>
      </body>
    </html>
  );
}
