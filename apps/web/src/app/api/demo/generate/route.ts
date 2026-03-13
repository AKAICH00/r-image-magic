import { cookies } from "next/headers";
import { NextRequest, NextResponse } from "next/server";

import { demoLimit } from "@/lib/demo-data";
import {
  getMeetMockupApiUrl,
  getPublicSiteUrl,
  isDemoApiConfigured,
} from "@/lib/env";

function getDemoErrorMessage(payload: unknown) {
  if (!payload || typeof payload !== "object") {
    return "Generation failed.";
  }

  const message =
    (payload as { error?: { message?: string } }).error?.message ??
    (payload as { message?: string }).message ??
    (payload as { error?: string }).error;

  return message ?? "Generation failed.";
}

export async function POST(request: NextRequest) {
  if (!isDemoApiConfigured()) {
    return NextResponse.json(
      {
        error:
          "The demo API key is not configured yet. Add `MEETMOCKUP_API_KEY` to enable live generation.",
      },
      { status: 500 },
    );
  }

  const cookieStore = await cookies();
  const countValue = cookieStore.get("demo_count")?.value;
  const currentCount = countValue ? Number.parseInt(countValue, 10) : 0;

  if (currentCount >= demoLimit) {
    return NextResponse.json(
      {
        error:
          "Demo limit reached for this session. Sign up for full access and 50 free mockups each month.",
      },
      { status: 429 },
    );
  }

  const body = (await request.json()) as {
    design_url: string;
    template_id: string;
    placement: { scale: number; offset_x: number; offset_y: number };
    options?: { tint_color?: string };
  };

  const publicSiteUrl = getPublicSiteUrl(request.nextUrl.origin);

  if (!publicSiteUrl && body.design_url.startsWith("/")) {
    return NextResponse.json(
      {
        error:
          "Relative demo assets need a public site URL. Set `NEXT_PUBLIC_SITE_URL` or test through a public domain or tunnel.",
      },
      { status: 400 },
    );
  }

  const resolvedDesignUrl = body.design_url.startsWith("/")
    ? `${publicSiteUrl}${body.design_url}`
    : body.design_url;

  const response = await fetch(
    `${getMeetMockupApiUrl()}/api/v1/mockups/generate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-API-Key": process.env.MEETMOCKUP_API_KEY ?? "",
      },
      body: JSON.stringify({
        ...body,
        design_url: resolvedDesignUrl,
      }),
      cache: "no-store",
    },
  );

  let payload: unknown;
  try {
    payload = await response.json();
  } catch {
    payload = null;
  }

  if (!response.ok) {
    return NextResponse.json(
      { error: getDemoErrorMessage(payload) },
      { status: response.status },
    );
  }

  const nextResponse = NextResponse.json(payload, { status: 200 });
  nextResponse.cookies.set("demo_count", String(currentCount + 1), {
    maxAge: 60 * 60,
    path: "/",
    sameSite: "lax",
    httpOnly: true,
  });

  return nextResponse;
}
