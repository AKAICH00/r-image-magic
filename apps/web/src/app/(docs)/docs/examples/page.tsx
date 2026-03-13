import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Code examples",
};

export default function ExamplesPage() {
  return (
    <article className="prose prose-neutral max-w-none rounded-[2rem] border border-black/8 bg-white/84 px-6 py-8 shadow-[0_18px_60px_rgba(15,18,24,0.06)]">
      <h1>Code examples</h1>

      <h2>Python</h2>
      <pre>
        <code>{`import requests

response = requests.post(
    "https://api.meetmockup.com/api/v1/mockups/generate",
    headers={
        "X-API-Key": "YOUR_API_KEY",
        "Content-Type": "application/json",
    },
    json={
        "design_url": "https://meetmockup.com/samples/sample-design-1.png",
        "template_id": "white_male_front",
        "placement": {"scale": 0.4, "offset_x": 0, "offset_y": -50},
    },
)

data = response.json()`}</code>
      </pre>

      <h2>Node.js</h2>
      <pre>
        <code>{`const response = await fetch(
  "https://api.meetmockup.com/api/v1/mockups/generate",
  {
    method: "POST",
    headers: {
      "X-API-Key": "YOUR_API_KEY",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      design_url: "https://meetmockup.com/samples/sample-design-1.png",
      template_id: "white_male_front",
      placement: { scale: 0.4, offset_x: 0, offset_y: -50 },
    }),
  }
);

const data = await response.json();`}</code>
      </pre>

      <h2>curl</h2>
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
    </article>
  );
}
