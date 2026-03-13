import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { ButtonLink } from "@/components/ui/button-link";

export default function BillingPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="font-display text-4xl font-semibold tracking-[-0.06em] text-foreground">
          Billing
        </h1>
        <p className="mt-2 text-muted-foreground">
          Paddle checkout and customer portal hooks are stubbed for v1.
        </p>
      </div>
      <div className="grid gap-4 lg:grid-cols-2">
        <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
          <CardHeader className="px-6 py-5">
            <CardTitle>Current plan</CardTitle>
          </CardHeader>
          <CardContent className="px-6 pb-6">
            <div className="font-display text-4xl font-semibold tracking-[-0.06em]">Free</div>
            <p className="mt-2 text-sm text-muted-foreground">
              50 mockups per month and starter template access.
            </p>
            <div className="mt-5">
              <ButtonLink href="/pricing" variant="outline">
                Upgrade plan
              </ButtonLink>
            </div>
          </CardContent>
        </Card>
        <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
          <CardHeader className="px-6 py-5">
            <CardTitle>Payment method</CardTitle>
          </CardHeader>
          <CardContent className="px-6 pb-6">
            <div className="flex h-32 items-center justify-center rounded-[1.5rem] border border-dashed border-black/10 bg-muted/60 text-sm text-muted-foreground">
              Managed by Paddle. Portal link coming soon.
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
