import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function DashboardPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="font-display text-4xl font-semibold tracking-[-0.06em] text-foreground">
          Dashboard
        </h1>
        <p className="mt-2 text-muted-foreground">
          Starter metrics and billing hooks for the full product experience.
        </p>
      </div>

      <div className="grid gap-4 lg:grid-cols-3">
        <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
          <CardHeader className="px-6 py-5">
            <CardTitle className="text-sm uppercase tracking-[0.18em] text-muted-foreground">
              Mockups this month
            </CardTitle>
          </CardHeader>
          <CardContent className="px-6 pb-6">
            <div className="font-display text-5xl font-semibold tracking-[-0.07em]">0 / 50</div>
          </CardContent>
        </Card>
        <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
          <CardHeader className="px-6 py-5">
            <CardTitle className="text-sm uppercase tracking-[0.18em] text-muted-foreground">
              Current plan
            </CardTitle>
          </CardHeader>
          <CardContent className="px-6 pb-6">
            <div className="font-display text-5xl font-semibold tracking-[-0.07em]">Free</div>
          </CardContent>
        </Card>
        <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
          <CardHeader className="px-6 py-5">
            <CardTitle className="text-sm uppercase tracking-[0.18em] text-muted-foreground">
              API keys
            </CardTitle>
          </CardHeader>
          <CardContent className="px-6 pb-6">
            <div className="font-display text-5xl font-semibold tracking-[-0.07em]">0</div>
          </CardContent>
        </Card>
      </div>

      <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
        <CardHeader className="px-6 py-5">
          <CardTitle>Usage over time</CardTitle>
        </CardHeader>
        <CardContent className="px-6 pb-6">
          <div className="flex h-64 items-center justify-center rounded-[1.5rem] border border-dashed border-black/10 bg-muted/60 text-sm text-muted-foreground">
            Usage chart placeholder for the next integration pass.
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
