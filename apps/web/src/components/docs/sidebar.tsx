"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

const docsNav = [
  { href: "/docs", label: "Getting started" },
  { href: "/docs/api-reference", label: "API reference" },
  { href: "/docs/examples", label: "Code examples" },
];

export function DocsSidebar() {
  const pathname = usePathname();

  return (
    <nav className="w-full shrink-0 lg:w-64">
      <div className="rounded-[1.8rem] border border-black/8 bg-white/80 p-4 lg:sticky lg:top-24">
        <h3 className="text-sm font-semibold uppercase tracking-[0.2em] text-muted-foreground">
          Documentation
        </h3>
        <ul className="mt-4 flex gap-2 overflow-x-auto pb-1 lg:flex-col">
          {docsNav.map((item) => {
            const active = pathname === item.href;

            return (
              <li key={item.href}>
                <Link
                  href={item.href}
                  className={`block rounded-2xl px-4 py-3 text-sm font-medium transition ${
                    active
                      ? "bg-[#151b26] text-white shadow-lg"
                      : "text-muted-foreground hover:bg-muted hover:text-foreground"
                  }`}
                >
                  {item.label}
                </Link>
              </li>
            );
          })}
        </ul>
      </div>
    </nav>
  );
}
