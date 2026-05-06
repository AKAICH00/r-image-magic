import { SignUp } from "@clerk/nextjs";

import { SelfServeSignup } from "@/components/marketing/self-serve-signup";
import { isClerkConfigured } from "@/lib/env";

export default function SignupPage() {
  if (!isClerkConfigured()) {
    return (
      <SelfServeSignup />
    );
  }

  return <SignUp />;
}
