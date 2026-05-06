"use client";

import { FormEvent, ReactNode, useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useApiKeySession } from "@/lib/api-key-session";

type ApiKeyGateProps = {
  title: string;
  description: string;
  children: (session: ReturnType<typeof useApiKeySession>) => ReactNode;
};

export function ApiKeyGate({ title, description, children }: ApiKeyGateProps) {
  const session = useApiKeySession();
  const [draftKey, setDraftKey] = useState("");

  useEffect(() => {
    if (session.ready) {
      setDraftKey(session.apiKey);
    }
  }, [session.apiKey, session.ready]);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    session.saveApiKey(draftKey);
  }

  if (!session.ready) {
    return (
      <div className="rounded-[1.9rem] border border-black/8 bg-white/86 px-6 py-8 text-sm text-muted-foreground">
        Loading dashboard session…
      </div>
    );
  }

  if (!session.apiKey) {
    return (
      <div className="rounded-[1.9rem] border border-black/8 bg-white/86 px-6 py-8 shadow-none">
        <div className="max-w-2xl">
          <h1 className="font-display text-4xl font-semibold tracking-[-0.06em] text-foreground">
            {title}
          </h1>
          <p className="mt-3 leading-7 text-muted-foreground">{description}</p>
          <form className="mt-8 space-y-4" onSubmit={handleSubmit}>
            <label className="block space-y-2">
              <span className="text-sm font-medium text-foreground">
                API key
              </span>
              <Input
                value={draftKey}
                onChange={(event) => setDraftKey(event.target.value)}
                placeholder="rim_..."
                className="h-11 rounded-2xl px-4"
              />
            </label>
            <div className="flex flex-wrap gap-3">
              <Button type="submit" className="rounded-full px-5">
                Save API key
              </Button>
            </div>
          </form>
          <p className="mt-4 text-sm text-muted-foreground">
            The key stays in your browser only. Use the signup flow if you
            still need a free developer key.
          </p>
        </div>
      </div>
    );
  }

  return <>{children(session)}</>;
}
