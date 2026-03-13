import { ApiShowcase } from "@/components/marketing/api-showcase";
import { BeforeAfter } from "@/components/marketing/before-after";
import { FinalCta } from "@/components/marketing/final-cta";
import { Hero } from "@/components/marketing/hero";
import { HowItWorks } from "@/components/marketing/how-it-works";
import { LiveDemoSection } from "@/components/marketing/live-demo-section";
import { PricingTeaser } from "@/components/marketing/pricing-teaser";
import { TemplateGalleryPreview } from "@/components/marketing/template-gallery-preview";
import { TrustStrip } from "@/components/marketing/trust-strip";

export default function HomePage() {
  return (
    <>
      <Hero />
      <BeforeAfter />
      <LiveDemoSection />
      <HowItWorks />
      <TemplateGalleryPreview />
      <ApiShowcase />
      <PricingTeaser />
      <TrustStrip />
      <FinalCta />
    </>
  );
}
