"use client";

import { useEffect, useState } from "react";

import { ApiKeyGate } from "@/components/dashboard/api-key-gate";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  ApiError,
  BillingSummaryResponse,
  fetchApiJson,
  useApiKeySession,
} from "@/lib/api-key-session";

export function DashboardBillingClient() {
  return (
    <ApiKeyGate
      title="Billing"
      description="Add an API key to see the backend’s current tier pricing, quota, and estimated monthly cost."
    >
      {(session) => <DashboardBillingContent session={session} />}
    </ApiKeyGate>
  );
}

function DashboardBillingContent({
  session,
}: {
  session: ReturnType<typeof useApiKeySession>;
}) {
  const [summary, setSummary] = useState<BillingSummaryResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let active = true;

    async function load() {
      setLoading(true);
      setError(null);

      try {
        const result = await fetchApiJson<BillingSummaryResponse>(
          "/api/v1/usage/billing",
          session.apiKey,
        );

        if (active) {
          setSummary(result);
        }
      } catch (err) {
        const apiError = err as ApiError;
        if (active) {
          setSummary(null);
          setError(apiError.message);
        }
      } finally {
        if (active) {
          setLoading(false);
        }
      }
    }

    void load();

    return () => {
      active = false;
    };
  }, [session.apiKey]);

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="font-display text-4xl font-semibold tracking-[-0.06em] text-foreground">
            Billing
          </h1>
          <p className="mt-2 text-muted-foreground">
            Pulled from the API pricing model instead of a frontend stub.
          </p>
        </div>
        <button
          type="button"
          onClick={session.clearApiKey}
          className="text-sm font-medium text-muted-foreground underline underline-offset-4"
        >
          Forget saved key
        </button>
      </div>

      {error ? (
        <Card className="rounded-[1.9rem] border border-amber-300 bg-amber-50/90 py-0 shadow-none">
          <CardContent className="px-6 py-5 text-sm text-amber-950">
            {error}
          </CardContent>
        </Card>
      ) : null}

      {loading || !summary ? (
        <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
          <CardContent className="px-6 py-8 text-sm text-muted-foreground">
            Loading billing summary…
          </CardContent>
        </Card>
      ) : (
        <>
          <div className="grid gap-4 lg:grid-cols-3">
            <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
              <CardHeader className="px-6 py-5">
                <CardTitle className="text-sm uppercase tracking-[0.18em] text-muted-foreground">
                  Current plan
                </CardTitle>
              </CardHeader>
              <CardContent className="px-6 pb-6">
                <div className="font-display text-5xl font-semibold tracking-[-0.07em] capitalize">
                  {summary.tier}
                </div>
                <p className="mt-2 text-sm text-muted-foreground">
                  {summary.tier_quota} monthly requests included.
                </p>
              </CardContent>
            </Card>
            <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
              <CardHeader className="px-6 py-5">
                <CardTitle className="text-sm uppercase tracking-[0.18em] text-muted-foreground">
                  Estimated cost
                </CardTitle>
              </CardHeader>
              <CardContent className="px-6 pb-6">
                <div className="font-display text-5xl font-semibold tracking-[-0.07em]">
                  ${summary.current_month.estimated_cost.toFixed(2)}
                </div>
                <p className="mt-2 text-sm text-muted-foreground">
                  Based on {summary.current_month.billable_requests} billable requests this month.
                </p>
              </CardContent>
            </Card>
            <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
              <CardHeader className="px-6 py-5">
                <CardTitle className="text-sm uppercase tracking-[0.18em] text-muted-foreground">
                  Overage
                </CardTitle>
              </CardHeader>
              <CardContent className="px-6 pb-6">
                <div className="font-display text-5xl font-semibold tracking-[-0.07em]">
                  {summary.current_month.overage_requests}
                </div>
                <p className="mt-2 text-sm text-muted-foreground">
                  ${summary.pricing.overage_price_per_1k.toFixed(2)} per extra 1k requests.
                </p>
              </CardContent>
            </Card>
          </div>

          <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
            <CardHeader className="px-6 py-5">
              <CardTitle>Billing details</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-3 px-6 pb-6 text-sm sm:grid-cols-2">
              <div>
                <div className="text-muted-foreground">Month</div>
                <div className="mt-1 font-medium text-foreground">
                  {summary.current_month.year_month}
                </div>
              </div>
              <div>
                <div className="text-muted-foreground">Base plan price</div>
                <div className="mt-1 font-medium text-foreground">
                  ${summary.pricing.tier_price.toFixed(2)}
                </div>
              </div>
              <div>
                <div className="text-muted-foreground">Currency</div>
                <div className="mt-1 font-medium text-foreground">
                  {summary.pricing.currency}
                </div>
              </div>
              <div>
                <div className="text-muted-foreground">Billable requests</div>
                <div className="mt-1 font-medium text-foreground">
                  {summary.current_month.billable_requests}
                </div>
              </div>
            </CardContent>
          </Card>
        </>
      )}
    </div>
  );
}
