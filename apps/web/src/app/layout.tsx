import "./globals.css";

export const metadata = {
  title: "IGY6 Local Evidence Workspace",
  description: "Local-first evidence, workflow, approval, and audit workspace"
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
