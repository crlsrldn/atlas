import { Handlers } from "$fresh/server.ts";
import { getCookies } from "$std/http/cookie.ts";
import { getSupabaseClient } from "../../utils/supabase.ts";
import Stripe from "npm:stripe@^14.0.0";

export const handler: Handlers = {
  async GET(req) {
    const kv = await Deno.openKv();
    const config = (await kv.get(["global_config"])).value as Record<string, unknown>;

    if (!config || !config.monetization_enabled || !config.stripe_secret_key) {
      return new Response("Monetization is not enabled.", { status: 400 });
    }

    const supabaseUrl = Deno.env.get("PUBLIC_SUPABASE_URL") || "";
    const supabaseAnonKey = Deno.env.get("PUBLIC_SUPABASE_ANON_KEY") || "";
    const supabase = getSupabaseClient(supabaseUrl, supabaseAnonKey);

    const cookies = getCookies(req.headers);
    const authHeader = req.headers.get("Authorization");
    
    // We can use session from cookies or Authorization header
    const token = cookies["sb-access-token"] || (authHeader ? authHeader.replace("Bearer ", "") : null);

    if (!token) {
      return new Response("Unauthorized", { status: 401 });
    }

    const { data: { user }, error: authError } = await supabase.auth.getUser(token);
    if (authError || !user) {
      return new Response("Unauthorized", { status: 401 });
    }

    const stripe = new Stripe(config.stripe_secret_key, {
      apiVersion: "2023-10-16",
    });

    const origin = new URL(req.url).origin;

    try {
      const session = await stripe.checkout.sessions.create({
        payment_method_types: ["card"],
        mode: "subscription",
        line_items: [
          {
            price_data: {
              currency: "usd",
              product_data: {
                name: "Atlas Premium",
                description: "Unlock 4K streaming and instant uncached downloads.",
              },
              unit_amount: 300, // $3.00/month
              recurring: {
                interval: "month",
              },
            },
            quantity: 1,
          },
        ],
        success_url: `${origin}/dashboard?upgrade=success`,
        cancel_url: `${origin}/dashboard?upgrade=canceled`,
        client_reference_id: user.id,
      });

      if (!session.url) {
        throw new Error("No session URL returned.");
      }

      return Response.redirect(session.url, 303);
    } catch (err) {
      console.error("Stripe error:", err);
      return new Response("Failed to create checkout session", { status: 500 });
    }
  },
};
