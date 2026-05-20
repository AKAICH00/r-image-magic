"use client";

import { useEffect, useState } from "react";

import { ApiKeyGate } from "@/components/dashboard/api-key-gate";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  ApiError,
  ApiKeyInfo,
  fetchApiJson,
  ListKeysResponse,
  UsageHistoryResponse,
  UsageStatsResponse,
  useApiKeySession,
} from "@/lib/api-key-session";

type DashboardState = {
  key: ApiKeyInfo;
  usage: UsageStatsResponse;
  history: UsageHistoryResponse;
  keys: ListKeysResponse;
};

function formatPercent(value: number) {
  return `${Math.round(value)}%`;
}

function formatMonthLabel(yearMonth: string) {
  const [year, month] = yearMonth.split("-");
  return new Date(Number(year), Number(month) - 1, 1).toLocaleDateString(
    "en-US",
    {
      month: "short",
      year: "numeric",
    },
  );
}

export function DashboardOverviewClient() {
  return (
    <ApiKeyGate
      title="Dashboard"
      description="Add an API key to see live usage, quotas, and the keys attached to the same owner email."
    >
      {(session) => <DashboardOverviewContent session={session} />}
    </ApiKeyGate>
  );
}

function DashboardOverviewContent({
  session,
}: {
  session: ReturnType<typeof useApiKeySession>;
}) {
  const [state, setState] = useState<DashboardState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    let active = true;

    async function load() {
      setLoading(true);
      setError(null);

      try {
        const [key, usage, history, keys] = await Promise.all([
          fetchApiJson<ApiKeyInfo>("/api/v1/keys/me", session.apiKey),
          fetchApiJson<UsageStatsResponse>("/api/v1/usage", session.apiKey),
          fetchApiJson<UsageHistoryResponse>(
            "/api/v1/usage/history?months=6",
            session.apiKey,
          ),
          fetchApiJson<ListKeysResponse>("/api/v1/keys", session.apiKey),
        ]);

        if (active) {
          setState({ key, usage, history, keys });
        }
      } catch (err) {
        const apiError = err as ApiError;
        if (active) {
          setState(null);
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
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h1 className="font-display text-4xl font-semibold tracking-[-0.06em] text-foreground">
            Dashboard
          </h1>
          <p className="mt-2 text-muted-foreground">
            Live API-key metrics for the developer-first onboarding flow.
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

      {loading || !state ? (
        <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
          <CardContent className="px-6 py-8 text-sm text-muted-foreground">
            Loading live usage data…
          </CardContent>
        </Card>
      ) : (
        <>
          <div className="grid gap-4 lg:grid-cols-3">
            <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
              <CardHeader className="px-6 py-5">
                <CardTitle className="text-sm uppercase tracking-[0.18em] text-muted-foreground">
                  Mockups this month
                </CardTitle>
              </CardHeader>
              <CardContent className="px-6 pb-6">
                <div className="font-display text-5xl font-semibold tracking-[-0.07em]">
                  {state.usage.quota.used} / {state.usage.quota.monthly_quota}
                </div>
                <p className="mt-2 text-sm text-muted-foreground">
                  {formatPercent(state.usage.quota.percentage_used)} of quota used in{" "}
                  {state.usage.current_month.year_month}.
                </p>
              </CardContent>
            </Card>
            <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
              <CardHeader className="px-6 py-5">
                <CardTitle className="text-sm uppercase tracking-[0.18em] text-muted-foreground">
                  Current plan
                </CardTitle>
              </CardHeader>
              <CardContent className="px-6 pb-6">
                <div className="font-display text-5xl font-semibold tracking-[-0.07em] capitalize">
                  {state.key.tier}
                </div>
                <p className="mt-2 text-sm text-muted-foreground">
                  {state.key.rate_limit_per_minute} requests per minute.
                </p>
              </CardContent>
            </Card>
            <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
              <CardHeader className="px-6 py-5">
                <CardTitle className="text-sm uppercase tracking-[0.18em] text-muted-foreground">
                  Active keys
                </CardTitle>
              </CardHeader>
              <CardContent className="px-6 pb-6">
                <div className="font-display text-5xl font-semibold tracking-[-0.07em]">
                  {state.keys.count}
                </div>
                <p className="mt-2 text-sm text-muted-foreground">
                  Owner email: {state.key.owner_email}
                </p>
              </CardContent>
            </Card>
          </div>

          <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
            <CardHeader className="px-6 py-5">
              <CardTitle>Recent monthly usage</CardTitle>
            </CardHeader>
            <CardContent className="px-6 pb-6">
              <div className="space-y-4">
                {state.history.months.length === 0 ? (
                  <div className="rounded-[1.5rem] border border-dashed border-black/10 bg-muted/60 px-4 py-10 text-sm text-muted-foreground">
                    No usage has been recorded for this key yet.
                  </div>
                ) : (
                  state.history.months.map((month) => (
                    <div key={month.year_month} className="space-y-2">
                      <div className="flex items-center justify-between gap-4 text-sm">
                        <span className="font-medium text-foreground">
                          {formatMonthLabel(month.year_month)}
                        </span>
                        <span className="text-muted-foreground">
                          {month.total_requests} total requests
                        </span>
                      </div>
                      <div className="h-3 overflow-hidden rounded-full bg-[#e8e1d0]">
                        <div
                          className="h-full rounded-full bg-[#151b26]"
                          style={{
                            width: `${Math.min(
                              100,
                              (month.total_requests /
                                state.usage.quota.monthly_quota) *
                                100,
                            )}%`,
                          }}
                        />
                      </div>
                      <div className="flex flex-wrap gap-4 text-xs text-muted-foreground">
                        <span>{month.successful_requests} successful</span>
                        <span>{month.failed_requests} failed</span>
                        <span>{month.overage_requests} overage</span>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </CardContent>
          </Card>
        </>
      )}
    </div>
  );
}
