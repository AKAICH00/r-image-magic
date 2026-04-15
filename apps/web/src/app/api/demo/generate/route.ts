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

type DemoGenerateBody = {
  design_url: string;
  template_id: string;
  placement: { scale: number; offset_x: number; offset_y: number };
  options?: { tint_color?: string };
};

function isFiniteNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function parseDemoGenerateBody(payload: unknown): DemoGenerateBody | string {
  if (!payload || typeof payload !== "object") {
    return "Request body must be a JSON object.";
  }

  const body = payload as Partial<DemoGenerateBody>;
  if (typeof body.design_url !== "string" || body.design_url.trim() === "") {
    return "design_url is required.";
  }
  if (typeof body.template_id !== "string" || body.template_id.trim() === "") {
    return "template_id is required.";
  }
  if (!body.placement || typeof body.placement !== "object") {
    return "placement is required.";
  }
  if (
    !isFiniteNumber(body.placement.scale) ||
    !isFiniteNumber(body.placement.offset_x) ||
    !isFiniteNumber(body.placement.offset_y)
  ) {
    return "placement scale, offset_x, and offset_y must be numbers.";
  }
  if (
    body.options?.tint_color !== undefined &&
    typeof body.options.tint_color !== "string"
  ) {
    return "options.tint_color must be a string.";
  }

  return {
    design_url: body.design_url,
    template_id: body.template_id,
    placement: {
      scale: body.placement.scale,
      offset_x: body.placement.offset_x,
      offset_y: body.placement.offset_y,
    },
    options: body.options,
  };
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

  let requestPayload: unknown;
  try {
    requestPayload = await request.json();
  } catch {
    return NextResponse.json(
      { error: "Request body must be valid JSON." },
      { status: 400 },
    );
  }

  const parsedBody = parseDemoGenerateBody(requestPayload);
  if (typeof parsedBody === "string") {
    return NextResponse.json({ error: parsedBody }, { status: 400 });
  }

  const publicSiteUrl = getPublicSiteUrl(request.nextUrl.origin);

  if (!publicSiteUrl && parsedBody.design_url.startsWith("/")) {
    return NextResponse.json(
      {
        error:
          "Relative demo assets need a public site URL. Set `NEXT_PUBLIC_SITE_URL` or test through a public domain or tunnel.",
      },
      { status: 400 },
    );
  }

  const resolvedDesignUrl = parsedBody.design_url.startsWith("/")
    ? `${publicSiteUrl}${parsedBody.design_url}`
    : parsedBody.design_url;

  const response = await fetch(
    `${getMeetMockupApiUrl()}/api/v1/mockups/generate`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-API-Key": process.env.MEETMOCKUP_API_KEY ?? "",
      },
      body: JSON.stringify({
        ...parsedBody,
        design_url: resolvedDesignUrl,
      }),
      cache: "no-store",
    },
  );

  let apiPayload: unknown;
  try {
    apiPayload = await response.json();
  } catch {
    apiPayload = null;
  }

  if (!response.ok) {
    return NextResponse.json(
      { error: getDemoErrorMessage(apiPayload) },
      { status: response.status },
    );
  }

  const nextResponse = NextResponse.json(apiPayload, { status: 200 });
  nextResponse.cookies.set("demo_count", String(currentCount + 1), {
    maxAge: 60 * 60,
    path: "/",
    sameSite: "lax",
    httpOnly: true,
  });

  return nextResponse;
}
