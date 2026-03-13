import { Footer } from "@/components/layout/footer";
import { Navbar } from "@/components/layout/navbar";

export default function MarketingLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <div className="page-shell flex min-h-screen flex-col">
      <Navbar />
      <main className="relative flex-1">{children}</main>
      <Footer />
    </div>
  );
}
