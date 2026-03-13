import fs from "node:fs";
import path from "node:path";
import { devices, expect, test } from "@playwright/test";

const baseURL = "http://127.0.0.1:3000";
const artifactRoot = "/tmp/meetmockup-qa-artifacts";

const desktopRoutes = [
  "/",
  "/pricing",
  "/demo",
  "/templates",
  "/docs",
  "/docs/api-reference",
  "/docs/examples",
  "/login",
  "/signup",
  "/dashboard",
  "/dashboard/keys",
  "/dashboard/billing",
  "/terms",
  "/privacy",
];

const mobileRoutes = ["/", "/demo", "/pricing", "/templates", "/docs"];

fs.rmSync(artifactRoot, { recursive: true, force: true });
fs.mkdirSync(artifactRoot, { recursive: true });

function slugify(route) {
  return route === "/" ? "home" : route.replace(/^\//, "").replace(/[\/?=&]+/g, "-");
}

function shouldIgnoreConsoleMessage(message) {
  return message.includes("Largest Contentful Paint");
}

async function inspectRoute(page, route, label) {
  const consoleMessages = [];
  const pageErrors = [];
  const failedRequests = [];

  page.on("console", (message) => {
    if (["error", "warning"].includes(message.type())) {
      const text = `[${message.type()}] ${message.text()}`;
      if (!shouldIgnoreConsoleMessage(text)) {
        consoleMessages.push(text);
      }
    }
  });

  page.on("pageerror", (error) => {
    pageErrors.push(error.message);
  });

  page.on("requestfailed", (request) => {
    failedRequests.push(`${request.method()} ${request.url()} :: ${request.failure()?.errorText}`);
  });

  const response = await page.goto(`${baseURL}${route}`, {
    waitUntil: "networkidle",
    timeout: 30000,
  });

  await expect(page).toHaveURL(
    new RegExp(
      route === "/"
        ? `${baseURL}/?$`
        : `${route.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}$`,
    ),
  );
  expect(response, `Missing response for ${route}`).not.toBeNull();
  expect(response.status(), `Unexpected status for ${route}`).toBeLessThan(400);

  await page.screenshot({
    path: path.join(artifactRoot, `${label}-${slugify(route)}.png`),
    fullPage: true,
  });

  return {
    route,
    status: response.status(),
    title: await page.title(),
    consoleMessages,
    pageErrors,
    failedRequests,
  };
}

test("desktop route sweep", async ({ browser }) => {
  const page = await browser.newPage({ viewport: { width: 1440, height: 1100 } });
  const results = [];

  for (const route of desktopRoutes) {
    results.push(await inspectRoute(page, route, "desktop"));
  }

  fs.writeFileSync(
    path.join(artifactRoot, "desktop-results.json"),
    JSON.stringify(results, null, 2),
  );

  for (const result of results) {
    expect(result.consoleMessages, `${result.route} had console errors`).toEqual([]);
    expect(result.pageErrors, `${result.route} had runtime errors`).toEqual([]);
    expect(result.failedRequests, `${result.route} had failed requests`).toEqual([]);
  }

  await page.close();
});

test("mobile route sweep", async ({ browser }) => {
  const context = await browser.newContext({ ...devices["iPhone SE"] });
  const page = await context.newPage();
  const results = [];

  for (const route of mobileRoutes) {
    results.push(await inspectRoute(page, route, "mobile"));
  }

  fs.writeFileSync(
    path.join(artifactRoot, "mobile-results.json"),
    JSON.stringify(results, null, 2),
  );

  for (const result of results) {
    expect(result.consoleMessages, `${result.route} had mobile console errors`).toEqual([]);
    expect(result.pageErrors, `${result.route} had mobile runtime errors`).toEqual([]);
    expect(result.failedRequests, `${result.route} had mobile failed requests`).toEqual([]);
  }

  await context.close();
});
