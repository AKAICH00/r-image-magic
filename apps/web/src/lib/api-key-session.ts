"use client";

import { useEffect, useState } from "react";

import { getMeetMockupApiUrl } from "@/lib/env";

const STORAGE_KEY = "meetmockup-api-key";

export type ApiError = {
  status: number;
  message: string;
};

export type ApiKeyInfo = {
  id: string;
  key_prefix: string;
  name: string;
  owner_email: string;
  owner_name?: string | null;
  company?: string | null;
  tier: string;
  rate_limit_per_minute: number;
  monthly_quota: number;
  is_active: boolean;
  created_at: string;
  last_used_at?: string | null;
  expires_at?: string | null;
};

export type UsageStatsResponse = {
  api_key_id: string;
  tier: string;
  current_month: {
    year_month: string;
    total_requests: number;
    successful_requests: number;
    failed_requests: number;
    billable_requests: number;
    overage_requests: number;
  };
  quota: {
    monthly_quota: number;
    used: number;
    remaining: number;
    percentage_used: number;
    is_exceeded: boolean;
  };
};

export type UsageHistoryResponse = {
  api_key_id: string;
  months: Array<{
    year_month: string;
    total_requests: number;
    successful_requests: number;
    failed_requests: number;
    billable_requests: number;
    overage_requests: number;
  }>;
};

export type BillingSummaryResponse = {
  api_key_id: string;
  tier: string;
  tier_quota: number;
  current_month: {
    year_month: string;
    billable_requests: number;
    overage_requests: number;
    estimated_cost: number;
  };
  pricing: {
    tier_price: number;
    overage_price_per_1k: number;
    currency: string;
  };
};

export type ListKeysResponse = {
  keys: ApiKeyInfo[];
  count: number;
};

function getStoredApiKey() {
  if (typeof window === "undefined") {
    return "";
  }

  return window.localStorage.getItem(STORAGE_KEY) ?? "";
}

export function useApiKeySession() {
  const [apiKey, setApiKey] = useState("");
  const [ready, setReady] = useState(false);

  useEffect(() => {
    setApiKey(getStoredApiKey());
    setReady(true);
  }, []);

  function saveApiKey(nextKey: string) {
    const normalized = nextKey.trim();
    setApiKey(normalized);

    if (typeof window !== "undefined") {
      if (normalized) {
        window.localStorage.setItem(STORAGE_KEY, normalized);
      } else {
        window.localStorage.removeItem(STORAGE_KEY);
      }
    }
  }

  function clearApiKey() {
    saveApiKey("");
  }

  return {
    apiKey,
    ready,
    saveApiKey,
    clearApiKey,
  };
}

export async function fetchApiJson<T>(
  path: string,
  apiKey: string,
  init?: RequestInit,
): Promise<T> {
  const response = await fetch(`${getMeetMockupApiUrl()}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(apiKey ? { "X-API-Key": apiKey } : {}),
      ...(init?.headers ?? {}),
    },
  });

  if (!response.ok) {
    const payload = (await response.json().catch(() => null)) as
      | { message?: string; error?: string }
      | null;

    throw {
      status: response.status,
      message:
        payload?.message ??
        payload?.error ??
        `Request failed with status ${response.status}`,
    } satisfies ApiError;
  }

  return (await response.json()) as T;
}
