export default function PrivacyPage() {
  return (
    <div className="py-16 sm:py-20">
      <div className="mx-auto max-w-3xl px-4 sm:px-6 lg:px-8">
        <article className="prose prose-neutral max-w-none rounded-[2rem] border border-black/8 bg-white/82 px-6 py-8 shadow-[0_18px_60px_rgba(15,18,24,0.06)]">
          <h1>Privacy</h1>
          <p>
            MeetMockup stores account, usage, and billing metadata required to run
            the service. Uploaded demo assets may be retained temporarily to render
            previews and debug failures.
          </p>
          <p>
            Production billing and tax handling will be managed through Paddle as
            merchant of record once subscriptions are enabled in the live product.
          </p>
        </article>
      </div>
    </div>
  );
}
