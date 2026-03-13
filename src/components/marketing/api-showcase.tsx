import { Badge } from "@/components/ui/badge";
import { ButtonLink } from "@/components/ui/button-link";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

const stats = [
  { value: "~1.5s", label: "Typical generation" },
  { value: "44", label: "Templates loaded" },
  { value: "REST", label: "OpenAPI documented" },
];

const examples = {
  curl: `curl -X POST https://api.meetmockup.com/api/v1/mockups/generate \\
  -H "X-API-Key: rim_your_key_here" \\
  -H "Content-Type: application/json" \\
  -d '{
    "design_url": "https://meetmockup.com/samples/sample-design-1.png",
    "template_id": "white_male_front",
    "placement": {
      "scale": 0.4,
      "offset_x": 0,
      "offset_y": -50
    }
  }'`,
  python: `import requests

response = requests.post(
    "https://api.meetmockup.com/api/v1/mockups/generate",
    headers={"X-API-Key": "rim_your_key_here"},
    json={
        "design_url": "https://meetmockup.com/samples/sample-design-1.png",
        "template_id": "white_male_front",
        "placement": {"scale": 0.4, "offset_x": 0, "offset_y": -50},
    },
)`,
  node: `const response = await fetch(
  "https://api.meetmockup.com/api/v1/mockups/generate",
  {
    method: "POST",
    headers: {
      "X-API-Key": "rim_your_key_here",
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      design_url: "https://meetmockup.com/samples/sample-design-1.png",
      template_id: "white_male_front",
      placement: { scale: 0.4, offset_x: 0, offset_y: -50 },
    }),
  }
);`,
};

export function ApiShowcase() {
  return (
    <section className="py-20 sm:py-24">
      <div className="mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
        <div className="overflow-hidden rounded-[2.2rem] border border-white/8 bg-[#0f1218] px-6 py-8 text-white shadow-[0_32px_120px_rgba(15,18,24,0.28)] sm:px-8 sm:py-10">
          <div className="grid gap-10 lg:grid-cols-[0.9fr_1.1fr]">
            <div className="flex flex-col justify-between">
              <div>
                <span className="kicker border-white/10 bg-white/5 text-white/62">
                  For developers
                </span>
                <h2 className="mt-5 font-display text-3xl font-semibold tracking-[-0.06em] text-white sm:text-4xl">
                  One API call. Realistic mockups.
                </h2>
                <p className="mt-4 text-lg leading-8 text-white/62">
                  Automate your product listings with a single POST request. Full
                  OpenAPI docs, sub-2s response times, and room to scale into
                  batch generation and webhooks.
                </p>
                <div className="mt-8 grid gap-4 sm:grid-cols-3">
                  {stats.map((stat) => (
                    <div
                      key={stat.label}
                      className="rounded-[1.6rem] border border-white/10 bg-white/5 px-4 py-4"
                    >
                      <div className="font-display text-3xl font-semibold tracking-[-0.06em] text-white">
                        {stat.value}
                      </div>
                      <div className="mt-2 text-sm text-white/52">{stat.label}</div>
                    </div>
                  ))}
                </div>
              </div>
              <div className="mt-8">
                <ButtonLink
                  href="/signup"
                  size="lg"
                  className="bg-[linear-gradient(135deg,var(--brand-signal),var(--brand-sun))] text-[#11151f] hover:opacity-90"
                >
                  Get your API key
                </ButtonLink>
              </div>
            </div>

            <div className="rounded-[1.9rem] border border-white/10 bg-white/5 p-4">
              <div className="mb-4 flex items-center gap-2 px-2">
                <span className="size-3 rounded-full bg-[#ff6b6b]" />
                <span className="size-3 rounded-full bg-[#ffd166]" />
                <span className="size-3 rounded-full bg-[#06d6a0]" />
                <Badge className="ml-auto border border-white/10 bg-white/10 text-white">
                  Swagger + examples
                </Badge>
              </div>
              <Tabs defaultValue="curl">
                <TabsList className="bg-white/6">
                  <TabsTrigger value="curl">curl</TabsTrigger>
                  <TabsTrigger value="python">Python</TabsTrigger>
                  <TabsTrigger value="node">Node.js</TabsTrigger>
                </TabsList>
                {Object.entries(examples).map(([key, value]) => (
                  <TabsContent key={key} value={key}>
                    <pre className="mt-4 overflow-x-auto rounded-[1.4rem] border border-white/10 bg-[#0b0e13] p-5 text-sm leading-7 text-white/78">
                      <code>{value}</code>
                    </pre>
                  </TabsContent>
                ))}
              </Tabs>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
