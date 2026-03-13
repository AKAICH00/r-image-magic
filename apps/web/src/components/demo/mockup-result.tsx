import { Badge } from "@/components/ui/badge";
import { ButtonLink } from "@/components/ui/button-link";

interface MockupResultProps {
  result: { mockupUrl: string; generationTimeMs: number; templateLabel: string } | null;
  isGenerating: boolean;
  error: string | null;
  noticeMessage?: string | null;
  compact?: boolean;
}

export function MockupResult({
  result,
  isGenerating,
  error,
  noticeMessage,
  compact = false,
}: MockupResultProps) {
  const frameClassName = compact ? "aspect-[4/3]" : "aspect-[4/5]";

  if (error) {
    return (
      <div className="section-frame flex min-h-96 flex-col items-center justify-center px-6 py-10 text-center">
        <div className="max-w-lg">
          <h3 className="text-xl font-semibold text-foreground">Demo unavailable</h3>
          <p className="mt-3 leading-7 text-muted-foreground">{error}</p>
          <div className="mt-6">
            <ButtonLink href="/signup">Sign up for API access</ButtonLink>
          </div>
        </div>
      </div>
    );
  }

  if (isGenerating) {
    return (
      <div className="section-frame flex min-h-96 items-center justify-center px-6 py-10">
        <div className="text-center">
          <div className="mx-auto size-9 animate-spin rounded-full border-3 border-[#151b26]/12 border-t-[#151b26]" />
          <p className="mt-4 text-sm font-medium text-muted-foreground">
            Generating your preview...
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="section-frame px-5 py-5">
      <div className={`relative overflow-hidden rounded-[1.6rem] bg-[#e7ebf1] ${frameClassName}`}>
        {result ? (
          <>
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img
              src={result.mockupUrl}
              alt={`Generated ${result.templateLabel} mockup`}
              className="h-full w-full object-contain"
            />
            <div className="absolute bottom-4 right-4 rounded-full bg-black/50 px-3 py-1 text-[0.72rem] font-semibold uppercase tracking-[0.2em] text-white/84 backdrop-blur">
              MeetMockup preview
            </div>
          </>
        ) : (
          <div className="flex h-full items-center justify-center px-8 text-center">
            <div>
              <p className="text-sm font-medium text-muted-foreground">
                {noticeMessage ?? "Your generated mockup will appear here."}
              </p>
              <p className="mt-2 text-sm text-muted-foreground">
                {noticeMessage
                  ? "Add the required env and refresh to enable live previews."
                  : "Choose a sample design to render the first result."}
              </p>
            </div>
          </div>
        )}
      </div>
      <div className="mt-4 flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="secondary" className="rounded-full">
            Preview quality
          </Badge>
          {result ? (
            <Badge variant="outline" className="rounded-full border-black/10">
              Generated in {(result.generationTimeMs / 1000).toFixed(1)}s
            </Badge>
          ) : null}
        </div>
        <div className="flex flex-wrap gap-3">
          {result ? (
            <a
              href={result.mockupUrl}
              download="meetmockup-preview.png"
              className="inline-flex h-8 items-center justify-center rounded-lg border border-black/10 bg-white px-3 text-sm font-medium text-foreground transition hover:bg-muted"
            >
              Download preview
            </a>
          ) : null}
          <ButtonLink href="/signup">Sign up for full quality</ButtonLink>
        </div>
      </div>
    </div>
  );
}
