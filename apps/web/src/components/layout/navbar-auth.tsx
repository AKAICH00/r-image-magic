"use client";

import { UserButton, useUser } from "@clerk/nextjs";

import { ButtonLink } from "@/components/ui/button-link";

export function NavbarAuth() {
  const { isLoaded, isSignedIn } = useUser();

  if (!isLoaded) {
    return null;
  }

  return (
    <>
      {!isSignedIn ? (
        <div className="hidden items-center gap-3 md:flex">
          <ButtonLink href="/login" variant="ghost" size="sm">
            Log in
          </ButtonLink>
          <ButtonLink
            href="/signup"
            size="sm"
            className="bg-[linear-gradient(135deg,var(--brand-signal),var(--brand-sun))] text-[#11151f] hover:opacity-90"
            >
              Sign up free
            </ButtonLink>
          </div>
      ) : (
        <div className="hidden items-center gap-3 md:flex">
          <ButtonLink href="/dashboard" variant="ghost" size="sm">
            Dashboard
          </ButtonLink>
          <UserButton />
        </div>
      )}
    </>
  );
}
