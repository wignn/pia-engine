import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "XAUUSD 4,429.825 USD — TradingView Pro Terminal",
  description: "Enterprise Grade Real-Time Financial Trading Terminal",
  icons: {
    icon: "/favicon.ico",
  }
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark">
      <body className="antialiased bg-[#131722] text-[#d1d4dc] h-screen w-screen overflow-hidden flex flex-col">
        {children}
      </body>
    </html>
  );
}
