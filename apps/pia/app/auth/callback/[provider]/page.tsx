"use client";

import { useEffect, useState } from "react";
import { useParams, useRouter, useSearchParams } from "next/navigation";
import { accountApi } from "@/src/lib/api/account";

export default function OAuthCallbackPage() {
  const params = useParams<{ provider: string }>();
  const router = useRouter();
  const searchParams = useSearchParams();
  const [error, setError] = useState("AUTHENTICATING...");

  useEffect(() => {
    const code = searchParams.get("code");
    const state = searchParams.get("state");
    if (!code || !state || !["google", "github"].includes(params.provider)) {
      setTimeout(() => setError("OAUTH CALLBACK IS INVALID"), 0);
      return;
    }

    accountApi.oauthCallback(params.provider, code, state)
      .then((result) => {
        if (result.error) throw new Error(result.error);
        router.replace("/portal/account");
      })
      .catch((reason: Error) => setError(reason.message || "OAUTH LOGIN FAILED"));
  }, [params.provider, router, searchParams]);

  return <main className="account-page"><div className="account-notice">{error}</div></main>;
}
