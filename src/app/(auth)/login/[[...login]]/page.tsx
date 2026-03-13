import { SignIn } from "@clerk/nextjs";

import { ButtonLink } from "@/components/ui/button-link";
import { isClerkConfigured } from "@/lib/env";

export default function LoginPage() {
  if (!isClerkConfigured()) {
    return (
      <div className="section-frame max-w-lg px-6 py-8 text-center">
        <h1 className="font-display text-3xl font-semibold tracking-[-0.06em]">
          Clerk is not configured
        </h1>
        <p className="mt-4 leading-7 text-muted-foreground">
          Add your Clerk publishable and secret keys to enable hosted sign-in.
        </p>
        <div className="mt-6">
          <ButtonLink href="/">Back to site</ButtonLink>
        </div>
      </div>
    );
  }

  return <SignIn />;
}
