import React, { useState } from "react";
import {
  Clock,
  Lightbulb,
  Tag,
  Search,
  Code2,
  Smartphone,
  CheckCircle2,
  Download,
} from "lucide-react";
import { usePayment } from "../../hooks/usePayment";
import { useUserPlan } from "../../hooks/useUserPlan";

const features = [
  { icon: Clock, title: "Unlimited Clipboard History", desc: "Never lose important snippets." },
  { icon: Lightbulb, title: "Pin Important Items", desc: "Keep frequently used snippets handy." },
  { icon: Tag, title: "Organise With Tags", desc: "Tag and filter your clips instantly." },
  { icon: Search, title: "Instant Search", desc: "Find any clip in milliseconds." },
  { icon: Code2, title: "Editable Clipboard", desc: "Edit any clipboard item before pasting." },
  { icon: Smartphone, title: "Cross-Platform Sync", desc: "Sync clips across all your devices." },
];

const FeatureCard = ({ icon: Icon, title, desc }) => (
  <div className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
    <div className="mb-3 flex h-11 w-11 items-center justify-center rounded-xl bg-blue-50">
      <Icon className="h-5 w-5 text-blue-600" />
    </div>
    <h3 className="text-base font-semibold text-slate-900">{title}</h3>
    <p className="mt-1.5 text-sm leading-6 text-slate-600">{desc}</p>
  </div>
);

export default function ClipTrayPricingPage() {
  const { openPaymentWebsite, isPolling, pollingError } = usePayment();
  const { refetchPlan } = useUserPlan();
  const [isProcessing, setIsProcessing] = useState(false);

  const handleUpgrade = async () => {
    setIsProcessing(true);
    const opened = await openPaymentWebsite();
    if (opened) {
      // Polling will start automatically
      // Refresh plan when polling detects payment
      const checkInterval = setInterval(async () => {
        if (!isPolling) {
          clearInterval(checkInterval);
          await refetchPlan();
          setIsProcessing(false);
        }
      }, 1000);
    } else {
      setIsProcessing(false);
    }
  };

  return (
    <div className="min-h-screen bg-white">
      <div className="mx-auto max-w-6xl px-4 py-10">
        <div className="grid gap-10 lg:grid-cols-2 lg:items-start">
          {/* Left */}
          <div>
            <div className="inline-flex items-center gap-2 rounded-full border border-slate-200 bg-white px-3 py-1 text-xs font-medium text-slate-700 shadow-sm">
              <span className="h-2 w-2 rounded-full bg-blue-600" />
              ClipTray Pricing
            </div>

            <h1 className="mt-5 text-4xl font-bold tracking-tight text-slate-900 sm:text-5xl">
              Lifetime access.
              <span className="block text-blue-600">One-time payment.</span>
            </h1>

            <p className="mt-4 max-w-xl text-base leading-7 text-slate-600">
              Unlock unlimited clipboard history & pinned clips, priority support, and all future
              features.
            </p>

            <div className="mt-7 grid max-w-xl grid-cols-1 gap-3 sm:grid-cols-2">
              {["Unlimited pinned clips", "Unlimited history", "Priority support", "Lifetime updates"].map(
                (t) => (
                  <div
                    key={t}
                    className="flex items-center gap-2 rounded-xl border border-slate-200 bg-white px-4 py-3 text-sm text-slate-700"
                  >
                    <CheckCircle2 className="h-5 w-5 text-blue-600" />
                    <span className="font-medium">{t}</span>
                  </div>
                )
              )}
            </div>
          </div>

          {/* Right: Pricing */}
          <div className="lg:sticky lg:top-10">
            <div className="relative overflow-hidden rounded-3xl bg-blue-600 shadow-xl">
              <div className="absolute left-1/2 top-4 -translate-x-1/2">
                <div className="rounded-full bg-yellow-300 px-4 py-1 text-xs font-extrabold tracking-wide text-slate-900">
                  BEST VALUE
                </div>
              </div>

              <div className="px-8 pb-8 pt-16 sm:px-10">
                <div className="text-white">
                  <div className="text-2xl font-semibold">Lifetime</div>

                  <div className="mt-6 flex items-end gap-3">
                    <div className="text-6xl font-extrabold tracking-tight">$29</div>
                    <div className="pb-2 text-lg font-medium opacity-90">one-time</div>
                  </div>

                  <div className="mt-9 space-y-3.5">
                    {[
                      "Unlimited pinned clips",
                      "Unlimited history",
                      "Everything in Free",
                      "Priority support",
                      "Lifetime updates",
                      "All future features",
                    ].map((item) => (
                      <div key={item} className="flex items-start gap-3">
                        <div className="mt-0.5 flex h-7 w-7 items-center justify-center rounded-full bg-white/15">
                          <CheckCircle2 className="h-5 w-5 text-white" />
                        </div>
                        <div className="text-base font-semibold">{item}</div>
                      </div>
                    ))}
                  </div>
                </div>

                <button
                  type="button"
                  onClick={handleUpgrade}
                  disabled={isPolling || isProcessing}
                  className="mt-8 inline-flex w-full items-center justify-center gap-2 rounded-2xl bg-white px-6 py-4 text-base font-bold text-blue-700 shadow-sm transition hover:bg-white/95 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <Download className="h-5 w-5" />
                  {isPolling || isProcessing ? "Processing..." : "Get Lifetime Access"}
                </button>

                <div className="mt-4 text-center text-xs text-white/80">
                  Secure checkout • Instant activation
                </div>
              </div>

              <div className="pointer-events-none absolute -right-16 -top-16 h-56 w-56 rounded-full bg-white/10" />
              <div className="pointer-events-none absolute -bottom-20 -left-20 h-72 w-72 rounded-full bg-white/10" />
            </div>
          </div>
        </div>

        {/* Short Features grid (no “See Features” button / no anchors) */}
        <div className="mt-12">
          <h2 className="text-xl font-bold text-slate-900">Included features</h2>
          <p className="mt-1 text-sm text-slate-600">Everything you need, included forever.</p>

          <div className="mt-5 grid gap-5 sm:grid-cols-2 lg:grid-cols-3">
            {features.map((f) => (
              <FeatureCard key={f.title} icon={f.icon} title={f.title} desc={f.desc} />
            ))}
          </div>
        </div>

        {/* Small footer CTA */}
        <div className="mt-12 flex flex-col items-center justify-between gap-3 rounded-3xl border border-slate-200 bg-white p-6 shadow-sm sm:flex-row">
          <div>
            <div className="text-base font-bold text-slate-900">Upgrade once, keep forever.</div>
            <div className="mt-1 text-sm text-slate-600">Lifetime access for $29.</div>
          </div>
          <button
            type="button"
            onClick={handleUpgrade}
            disabled={isPolling || isProcessing}
            className="inline-flex items-center justify-center rounded-2xl bg-blue-600 px-6 py-3 text-sm font-semibold text-white shadow-sm transition hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isPolling || isProcessing ? "Processing..." : "Get Lifetime Access — $29"}
          </button>
        </div>
      </div>
      {pollingError && (
        <div className="mt-4 mx-auto max-w-6xl px-4">
          <div className="rounded-lg border border-red-200 bg-red-50 p-4 text-sm text-red-700">
            {pollingError}
          </div>
        </div>
      )}
    </div>
  );
}
