"use client";

import { useEffect, useState } from "react";

import { ApiKeyGate } from "@/components/dashboard/api-key-gate";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import {
  ApiError,
  ApiKeyInfo,
  fetchApiJson,
  ListKeysResponse,
  useApiKeySession,
} from "@/lib/api-key-session";

type KeysState = {
  currentKey: ApiKeyInfo;
  keys: ApiKeyInfo[];
};

export function DashboardKeysClient() {
  return (
    <ApiKeyGate
      title="API Keys"
      description="Save an API key to inspect the current owner, rotate between keys, or revoke stale keys."
    >
      {(session) => <DashboardKeysContent session={session} />}
    </ApiKeyGate>
  );
}

function DashboardKeysContent({
  session,
}: {
  session: ReturnType<typeof useApiKeySession>;
}) {
  const [state, setState] = useState<KeysState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [draftKey, setDraftKey] = useState(session.apiKey);
  const [revokingId, setRevokingId] = useState<string | null>(null);

  useEffect(() => {
    setDraftKey(session.apiKey);
  }, [session.apiKey]);

  useEffect(() => {
    let active = true;

    async function load() {
      setLoading(true);
      setError(null);

      try {
        const [currentKey, keysResponse] = await Promise.all([
          fetchApiJson<ApiKeyInfo>("/api/v1/keys/me", session.apiKey),
          fetchApiJson<ListKeysResponse>("/api/v1/keys", session.apiKey),
        ]);

        if (active) {
          setState({ currentKey, keys: keysResponse.keys });
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

  async function handleRevoke(keyId: string) {
    setRevokingId(keyId);
    setError(null);

    try {
      await fetchApiJson<{ message: string }>(`/api/v1/keys/${keyId}`, session.apiKey, {
        method: "DELETE",
      });

      if (state?.currentKey.id === keyId) {
        session.clearApiKey();
        return;
      }

      const keysResponse = await fetchApiJson<ListKeysResponse>("/api/v1/keys", session.apiKey);
      setState((current) =>
        current ? { ...current, keys: keysResponse.keys } : current,
      );
    } catch (err) {
      const apiError = err as ApiError;
      setError(apiError.message);
    } finally {
      setRevokingId(null);
    }
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="font-display text-4xl font-semibold tracking-[-0.06em] text-foreground">
            API Keys
          </h1>
          <p className="mt-2 text-muted-foreground">
            The dashboard uses a saved key to talk directly to the Rust API.
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

      <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
        <CardHeader className="px-6 py-5">
          <CardTitle>Saved browser key</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4 px-6 pb-6">
          <Input
            value={draftKey}
            onChange={(event) => setDraftKey(event.target.value)}
            className="h-11 rounded-2xl px-4"
          />
          <div className="flex flex-wrap gap-3">
            <Button
              type="button"
              className="rounded-full px-5"
              onClick={() => session.saveApiKey(draftKey)}
            >
              Replace saved key
            </Button>
          </div>
        </CardContent>
      </Card>

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
            Loading key inventory…
          </CardContent>
        </Card>
      ) : (
        <>
          <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
            <CardHeader className="px-6 py-5">
              <CardTitle>Current key identity</CardTitle>
            </CardHeader>
            <CardContent className="grid gap-3 px-6 pb-6 text-sm sm:grid-cols-2">
              <div>
                <div className="text-muted-foreground">Owner email</div>
                <div className="mt-1 font-medium text-foreground">
                  {state.currentKey.owner_email}
                </div>
              </div>
              <div>
                <div className="text-muted-foreground">Key prefix</div>
                <div className="mt-1 font-mono text-foreground">
                  {state.currentKey.key_prefix}...
                </div>
              </div>
              <div>
                <div className="text-muted-foreground">Tier</div>
                <div className="mt-1 font-medium capitalize text-foreground">
                  {state.currentKey.tier}
                </div>
              </div>
              <div>
                <div className="text-muted-foreground">Rate limit</div>
                <div className="mt-1 font-medium text-foreground">
                  {state.currentKey.rate_limit_per_minute} req/min
                </div>
              </div>
            </CardContent>
          </Card>

          <Card className="rounded-[1.9rem] border border-black/8 bg-white/86 py-0 shadow-none">
            <CardHeader className="px-6 py-5">
              <CardTitle>Keys on this owner account</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4 px-6 pb-6">
              {state.keys.map((key) => (
                <div
                  key={key.id}
                  className="flex flex-col gap-4 rounded-[1.5rem] border border-black/8 bg-[#fbfaf7] px-4 py-4 sm:flex-row sm:items-center sm:justify-between"
                >
                  <div>
                    <div className="font-medium text-foreground">{key.name}</div>
                    <div className="mt-1 text-sm text-muted-foreground">
                      {key.key_prefix}... · {key.owner_email} ·{" "}
                      <span className="capitalize">{key.tier}</span>
                    </div>
                  </div>
                  <div className="flex items-center gap-3">
                    <span className="text-xs uppercase tracking-[0.18em] text-muted-foreground">
                      {key.is_active ? "Active" : "Revoked"}
                    </span>
                    {key.is_active ? (
                      <Button
                        type="button"
                        variant="outline"
                        className="rounded-full px-4"
                        disabled={revokingId === key.id}
                        onClick={() => handleRevoke(key.id)}
                      >
                        {revokingId === key.id ? "Revoking…" : "Revoke"}
                      </Button>
                    ) : null}
                  </div>
                </div>
              ))}
            </CardContent>
          </Card>
        </>
      )}
    </div>
  );
}
