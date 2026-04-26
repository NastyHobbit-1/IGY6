import "./globals.css";

export const metadata = {
  title: "IGY6 Phase 0",
  description: "Adaptive Intelligence System skeleton status"
};

export default function RootLayout({
  children
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
