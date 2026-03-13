import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ButtonLink } from "@/components/ui/button-link";

export default function ApiKeysPage() {
  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="font-display text-4xl font-semibold tracking-[-0.06em] text-foreground">
            API Keys
          </h1>
          <p className="mt-2 text-muted-foreground">
            Key management will connect to the backend key endpoints in the next pass.
          </p>
        </div>
        <ButtonLink href="/signup">Create API key</ButtonLink>
      </div>

      <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
        <CardHeader className="px-6 py-5">
          <CardTitle>Your keys</CardTitle>
        </CardHeader>
        <CardContent className="px-6 pb-6">
          <div className="flex h-40 items-center justify-center rounded-[1.5rem] border border-dashed border-black/10 bg-muted/60 text-sm text-muted-foreground">
            No API keys yet. This stub is ready for the backend wiring.
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
