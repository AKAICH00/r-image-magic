const signals = [
  "Sub-2s generation",
  "99.9% uptime target",
  "44 live templates",
  "9 product types",
  "OpenAPI documented",
];

export function TrustStrip() {
  return (
    <section className="border-y border-black/8 bg-[#f2ede3] py-7">
      <div className="mx-auto flex max-w-7xl flex-wrap items-center justify-center gap-x-8 gap-y-3 px-4 text-sm font-semibold uppercase tracking-[0.22em] text-muted-foreground sm:px-6 lg:px-8">
        {signals.map((signal) => (
          <span key={signal}>{signal}</span>
        ))}
      </div>
    </section>
  );
}
