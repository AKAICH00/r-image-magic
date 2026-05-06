"use client";

import { FormEvent, useMemo, useState } from "react";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { fetchApiJson, useApiKeySession } from "@/lib/api-key-session";

type SignupResponse = {
  id: string;
  api_key: string;
  key_prefix: string;
  tier: string;
  rate_limit_per_minute: number;
  monthly_quota: number;
  owner_email: string;
  message: string;
};

export function SelfServeSignup() {
  const { saveApiKey } = useApiKeySession();
  const [email, setEmail] = useState("");
  const [name, setName] = useState("");
  const [company, setCompany] = useState("");
  const [projectName, setProjectName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<SignupResponse | null>(null);

  const canSubmit = useMemo(() => email.trim().length > 0, [email]);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const response = await fetchApiJson<SignupResponse>(
        "/api/v1/keys/signup",
        "",
        {
          method: "POST",
          body: JSON.stringify({
            email,
            name,
            company,
            project_name: projectName,
          }),
        },
      );

      saveApiKey(response.api_key);
      setResult(response);
    } catch (err) {
      const message =
        err instanceof Error
          ? err.message
          : typeof err === "object" && err && "message" in err
            ? String((err as { message: string }).message)
            : "Failed to create API key";
      setError(message);
    } finally {
      setLoading(false);
    }
  }

  if (result) {
    return (
      <Card className="section-frame max-w-2xl border border-black/8 bg-white/90 py-0 shadow-[0_18px_60px_rgba(15,18,24,0.06)]">
        <CardHeader className="px-6 py-6">
          <CardTitle className="font-display text-3xl font-semibold tracking-[-0.06em]">
            Free API key created
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-5 px-6 pb-6">
          <p className="leading-7 text-muted-foreground">
            This is the only time the full key will be shown. It has also been
            saved locally in this browser for the dashboard pages.
          </p>
          <div className="rounded-[1.5rem] border border-black/8 bg-[#fbfaf7] p-4">
            <div className="text-sm text-muted-foreground">API key</div>
            <code className="mt-2 block overflow-x-auto text-sm text-foreground">
              {result.api_key}
            </code>
          </div>
          <div className="grid gap-4 sm:grid-cols-3">
            <div className="rounded-[1.4rem] border border-black/8 bg-[#fbfaf7] p-4">
              <div className="text-sm text-muted-foreground">Tier</div>
              <div className="mt-1 font-medium capitalize text-foreground">
                {result.tier}
              </div>
            </div>
            <div className="rounded-[1.4rem] border border-black/8 bg-[#fbfaf7] p-4">
              <div className="text-sm text-muted-foreground">Rate limit</div>
              <div className="mt-1 font-medium text-foreground">
                {result.rate_limit_per_minute} req/min
              </div>
            </div>
            <div className="rounded-[1.4rem] border border-black/8 bg-[#fbfaf7] p-4">
              <div className="text-sm text-muted-foreground">Monthly quota</div>
              <div className="mt-1 font-medium text-foreground">
                {result.monthly_quota}
              </div>
            </div>
          </div>
          <div className="flex flex-wrap gap-3">
            <Link
              href="/dashboard"
              className="inline-flex h-11 items-center justify-center rounded-full bg-[#151b26] px-5 text-sm font-medium text-white transition hover:opacity-92"
            >
              Open dashboard
            </Link>
            <Link
              href="/docs"
              className="inline-flex h-11 items-center justify-center rounded-full border border-black/10 bg-white px-5 text-sm font-medium text-foreground transition hover:bg-muted"
            >
              Read the docs
            </Link>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="section-frame max-w-2xl border border-black/8 bg-white/90 py-0 shadow-[0_18px_60px_rgba(15,18,24,0.06)]">
      <CardHeader className="px-6 py-6">
        <CardTitle className="font-display text-3xl font-semibold tracking-[-0.06em]">
          Create a free developer key
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-5 px-6 pb-6">
        <p className="leading-7 text-muted-foreground">
          This path provisions a free-tier API key directly from the backend so
          the product stays usable even before full account billing is wired in.
        </p>
        <form className="space-y-4" onSubmit={handleSubmit}>
          <label className="block space-y-2">
            <span className="text-sm font-medium text-foreground">Email</span>
            <Input
              type="email"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              placeholder="you@store.com"
              className="h-11 rounded-2xl px-4"
            />
          </label>
          <div className="grid gap-4 sm:grid-cols-2">
            <label className="block space-y-2">
              <span className="text-sm font-medium text-foreground">Name</span>
              <Input
                value={name}
                onChange={(event) => setName(event.target.value)}
                placeholder="Alex"
                className="h-11 rounded-2xl px-4"
              />
            </label>
            <label className="block space-y-2">
              <span className="text-sm font-medium text-foreground">Company</span>
              <Input
                value={company}
                onChange={(event) => setCompany(event.target.value)}
                placeholder="Northwind Studio"
                className="h-11 rounded-2xl px-4"
              />
            </label>
          </div>
          <label className="block space-y-2">
            <span className="text-sm font-medium text-foreground">
              Project name
            </span>
            <Input
              value={projectName}
              onChange={(event) => setProjectName(event.target.value)}
              placeholder="Etsy hoodie launch"
              className="h-11 rounded-2xl px-4"
            />
          </label>
          {error ? (
            <div className="rounded-[1.2rem] border border-amber-300 bg-amber-50 px-4 py-3 text-sm text-amber-950">
              {error}
            </div>
          ) : null}
          <div className="flex flex-wrap gap-3">
            <Button
              type="submit"
              className="rounded-full px-5"
              disabled={!canSubmit || loading}
            >
              {loading ? "Creating key…" : "Create free key"}
            </Button>
            <Link
              href="/docs"
              className="inline-flex h-11 items-center justify-center rounded-full border border-black/10 bg-white px-5 text-sm font-medium text-foreground transition hover:bg-muted"
            >
              View API docs
            </Link>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
