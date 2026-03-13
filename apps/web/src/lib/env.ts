const LOCAL_HOST_PATTERNS = ["localhost", "127.0.0.1", "0.0.0.0"];

export function isClerkConfigured() {
  return Boolean(
    process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY && process.env.CLERK_SECRET_KEY,
  );
}

export function isDemoApiConfigured() {
  return Boolean(process.env.MEETMOCKUP_API_KEY);
}

export function isDemoReadyForRelativeAssets() {
  return isDemoApiConfigured() && Boolean(getPublicSiteUrl());
}

export function getMeetMockupApiUrl() {
  return process.env.MEETMOCKUP_API_URL ?? "https://api.meetmockup.com";
}

export function normalizeBaseUrl(url: string) {
  return url.replace(/\/$/, "");
}

export function isPublicOrigin(origin: string) {
  try {
    const { hostname } = new URL(origin);
    return !LOCAL_HOST_PATTERNS.includes(hostname);
  } catch {
    return false;
  }
}

export function getPublicSiteUrl(origin?: string) {
  const configured = process.env.NEXT_PUBLIC_SITE_URL;

  if (configured) {
    return isPublicOrigin(configured) ? normalizeBaseUrl(configured) : null;
  }

  if (origin && isPublicOrigin(origin)) {
    return normalizeBaseUrl(origin);
  }

  return null;
}
