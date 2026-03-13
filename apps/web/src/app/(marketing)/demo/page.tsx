import type { Metadata } from "next";

import { DemoPage } from "@/components/demo/demo-page";

export const metadata: Metadata = {
  title: "Try the demo",
  description:
    "Upload a design, choose a template, and generate a realistic product mockup with MeetMockup.",
};

export default function DemoRoute() {
  return <DemoPage />;
}
