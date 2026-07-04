import { useEffect, useState } from "preact/hooks";

export default function AdminSettings() {
  const [monetizationEnabled, setMonetizationEnabled] = useState(false);
  const [stripeSecretKey, setStripeSecretKey] = useState("");
  const [stripeWebhookSecret, setStripeWebhookSecret] = useState("");
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  useEffect(() => {
    fetch("/api/global-config")
      .then((res) => res.json())
      .then((data) => {
        setMonetizationEnabled(data.monetization_enabled === true);
        setStripeSecretKey(data.stripe_secret_key || "");
        setStripeWebhookSecret(data.stripe_webhook_secret || "");
        setLoading(false);
      })
      .catch((e) => {
        console.error("Failed to fetch global config", e);
        setLoading(false);
      });
  }, []);

  const saveConfig = async () => {
    setSaving(true);
    setSaveError(null);
    setSaveSuccess(false);

    try {
      const res = await fetch("/api/global-config", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          monetization_enabled: monetizationEnabled,
          stripe_secret_key: stripeSecretKey,
          stripe_webhook_secret: stripeWebhookSecret,
        }),
      });

      if (!res.ok) {
        throw new Error("Failed to save config");
      }

      setSaveSuccess(true);
      setTimeout(() => setSaveSuccess(false), 3000);
    } catch (err) {
      setSaveError(err instanceof Error ? err.message : "An error occurred");
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div class="glass-card-strong p-8 rounded-2xl border border-zinc-200/50 dark:border-white/5 space-y-6">
        <div class="h-6 w-48 shimmer rounded" />
        <div class="h-10 shimmer rounded-xl" />
        <div class="h-10 shimmer rounded-xl" />
      </div>
    );
  }

  return (
    <div class="glass-card-strong p-8 rounded-2xl border border-zinc-200/50 dark:border-white/5 shadow-xl shadow-black/5 dark:shadow-black/20">
      <div class="flex items-center justify-between mb-6">
        <div>
          <h2 class="text-xl font-bold text-zinc-900 dark:text-white flex items-center gap-2">
            <svg
              class="w-5 h-5 text-indigo-500"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            Monetization Settings
          </h2>
          <p class="text-sm text-zinc-500 dark:text-zinc-400 mt-1">
            Configure premium tier limits and Stripe integration.
          </p>
        </div>
      </div>

      <div class="space-y-6">
        {saveError && (
          <div class="p-4 bg-red-50 dark:bg-red-500/10 border border-red-200 dark:border-red-500/20 rounded-xl text-red-600 dark:text-red-400 text-sm">
            {saveError}
          </div>
        )}

        <div class="space-y-2">
          <p class="block text-sm font-medium text-zinc-700 dark:text-zinc-300">
            Enable Monetization
          </p>
          <button
            type="button"
            role="switch"
            aria-checked={monetizationEnabled}
            onClick={() => setMonetizationEnabled(!monetizationEnabled)}
            class="toggle-wrapper group"
          >
            <div class={`toggle-track ${monetizationEnabled ? "active" : ""}`}>
              <div class="toggle-thumb" />
            </div>
            <div class="text-left">
              <span class="text-sm font-medium text-zinc-700 group-hover:text-zinc-900 dark:text-zinc-300 dark:group-hover:text-white transition-colors block">
                {monetizationEnabled ? "Enabled" : "Disabled"}
              </span>
              <span class="text-xs text-zinc-500 dark:text-zinc-500 block mt-0.5">
                {monetizationEnabled
                  ? "Free users are restricted to 1080p and cannot use uncached TorBox streams."
                  : "All users have full access to 4K and uncached streams."}
              </span>
            </div>
          </button>
        </div>

        <div class="space-y-4">
          <div class="space-y-1.5">
            <label class="block text-sm font-medium text-zinc-700 dark:text-zinc-300">
              Stripe Secret Key
            </label>
            <input
              type="password"
              value={stripeSecretKey}
              onChange={(e) =>
                setStripeSecretKey((e.target as HTMLInputElement).value)}
              placeholder="sk_test_..."
              class="input-field font-mono text-sm"
            />
          </div>

          <div class="space-y-1.5">
            <label class="block text-sm font-medium text-zinc-700 dark:text-zinc-300">
              Stripe Webhook Secret
            </label>
            <input
              type="password"
              value={stripeWebhookSecret}
              onChange={(e) =>
                setStripeWebhookSecret((e.target as HTMLInputElement).value)}
              placeholder="whsec_..."
              class="input-field font-mono text-sm"
            />
            <p class="text-xs text-zinc-500 dark:text-zinc-500 mt-1">
              Required to receive successful checkout events. Set this up in
              your Stripe Dashboard pointing to `/api/stripe-webhook`.
            </p>
          </div>
        </div>

        <div class="pt-4 flex items-center justify-between border-t border-zinc-200/50 dark:border-white/5">
          {saveSuccess
            ? (
              <span class="text-sm font-medium text-emerald-600 dark:text-emerald-400 flex items-center gap-1.5">
                <svg
                  class="w-4 h-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M5 13l4 4L19 7"
                  />
                </svg>
                Saved successfully
              </span>
            )
            : <div />}
          <button
            type="button"
            onClick={saveConfig}
            disabled={saving}
            class="btn-primary flex items-center gap-2 px-6"
          >
            {saving
              ? (
                <>
                  <div class="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                  Saving...
                </>
              )
              : (
                "Save Settings"
              )}
          </button>
        </div>
      </div>
    </div>
  );
}
