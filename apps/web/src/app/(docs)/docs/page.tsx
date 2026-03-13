import type { Metadata } from "next";
import Link from "next/link";

export const metadata: Metadata = {
  title: "Getting started",
};

export default function DocsGettingStartedPage() {
  return (
    <article className="prose prose-neutral max-w-none rounded-[2rem] border border-black/8 bg-white/84 px-6 py-8 shadow-[0_18px_60px_rgba(15,18,24,0.06)]">
      <h1>Getting started</h1>
      <p>
        MeetMockup generates photorealistic mockups with displacement mapping.
        This guide walks through the fastest path to your first API request.
      </p>

      <h2>1. Create an account</h2>
      <p>
        Sign up at <Link href="/signup">meetmockup.com/signup</Link> to get
        access to your account area and API credentials.
      </p>

      <h2>2. Make your first request</h2>
      <pre>
        <code>{`curl -X POST https://api.meetmockup.com/api/v1/mockups/generate \\
  -H "X-API-Key: YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "design_url": "https://meetmockup.com/samples/sample-design-1.png",
    "template_id": "white_male_front",
    "placement": {
      "scale": 0.4,
      "offset_x": 0,
      "offset_y": -50
    }
  }'`}</code>
      </pre>

      <h2>3. Handle the response</h2>
      <p>
        The API returns a base64-encoded PNG in <code>mockup_url</code> alongside
        metadata including generation time and output dimensions.
      </p>
      <pre>
        <code>{`{
  "success": true,
  "mockup_url": "data:image/png;base64,iVBORw0KGgo...",
  "metadata": {
    "generation_time_ms": 1450,
    "template_used": "white_male_front",
    "dimensions": {
      "width": 2400,
      "height": 3200
    }
  }
}`}</code>
      </pre>

      <h2>4. Explore the catalog</h2>
      <p>
        Browse templates visually on{" "}
        <Link href="/templates">the templates page</Link> or integrate the
        listing endpoints directly from your backend.
      </p>
    </article>
  );
}
