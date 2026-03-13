import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "API reference",
};

export default function ApiReferencePage() {
  return (
    <article className="prose prose-neutral max-w-none rounded-[2rem] border border-black/8 bg-white/84 px-6 py-8 shadow-[0_18px_60px_rgba(15,18,24,0.06)]">
      <h1>API reference</h1>
      <p>
        MeetMockup exposes a REST API with OpenAPI documentation and predictable
        JSON responses.
      </p>

      <h2>Base URL</h2>
      <pre>
        <code>https://api.meetmockup.com</code>
      </pre>

      <h2>Authentication</h2>
      <p>
        Send your API key in the <code>X-API-Key</code> header.
      </p>

      <h2>Key endpoints</h2>
      <table>
        <thead>
          <tr>
            <th>Method</th>
            <th>Path</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>POST</td>
            <td>/api/v1/mockups/generate</td>
            <td>Generate a mockup</td>
          </tr>
          <tr>
            <td>GET</td>
            <td>/api/v1/templates</td>
            <td>List all templates</td>
          </tr>
          <tr>
            <td>GET</td>
            <td>/api/v1/templates/product-types</td>
            <td>List product types and counts</td>
          </tr>
          <tr>
            <td>GET</td>
            <td>/api/v1/usage</td>
            <td>Usage stats for the current key</td>
          </tr>
          <tr>
            <td>GET</td>
            <td>/health</td>
            <td>Service health check</td>
          </tr>
        </tbody>
      </table>

      <h2>Interactive docs</h2>
      <p>
        Open the live Swagger reference at{" "}
        <a href="https://api.meetmockup.com/swagger-ui/" target="_blank" rel="noreferrer">
          api.meetmockup.com/swagger-ui/
        </a>
        .
      </p>
    </article>
  );
}
