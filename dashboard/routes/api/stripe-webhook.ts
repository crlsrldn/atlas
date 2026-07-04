import { Handlers } from "$fresh/server.ts";
import { getAdminSupabaseClient } from "../../utils/admin_supabase.ts";
import Stripe from "npm:stripe@^14.0.0";

export const handler: Handlers = {
  async POST(req) {
    const kv = await Deno.openKv();
    const config = (await kv.get(["global_config"])).value as Record<
      string,
      unknown
    >;

    if (!config || !config.stripe_secret_key || !config.stripe_webhook_secret) {
      return new Response("Webhook not configured", { status: 400 });
    }

    const stripe = new Stripe(config.stripe_secret_key as string, {
      apiVersion: "2023-10-16",
    });

    const signature = req.headers.get("stripe-signature");
    if (!signature) {
      return new Response("Missing signature", { status: 400 });
    }

    const body = await req.text();
    let event: Stripe.Event;

    try {
      event = stripe.webhooks.constructEvent(
        body,
        signature,
        config.stripe_webhook_secret as string,
      );
    } catch (err) {
      console.error("Webhook signature verification failed.", err);
      return new Response("Webhook Error", { status: 400 });
    }

    const supabase = getAdminSupabaseClient();
    if (!supabase) {
      return new Response("Supabase admin client missing", { status: 500 });
    }

    try {
      switch (event.type) {
        case "checkout.session.completed": {
          const session = event.data.object as Stripe.Checkout.Session;
          const userId = session.client_reference_id;

          if (userId) {
            // First fetch the existing preferences JSON
            const { data: prefData } = await supabase
              .from("preferences")
              .select("prefs_json")
              .eq("id", userId)
              .single();

            const prefs = prefData?.prefs_json || {};
            prefs.is_premium = true;
            prefs.stripe_customer_id = session.customer;
            prefs.stripe_subscription_id = session.subscription;

            // Upsert the updated preferences
            await supabase
              .from("preferences")
              .upsert({ id: userId, prefs_json: prefs });
          }
          break;
        }
        case "customer.subscription.deleted": {
          const subscription = event.data.object as Stripe.Subscription;
          const customerId = subscription.customer;

          // Find the user with this customer ID
          // Since we store it in a JSON column, we have to query all or use Postgres JSON querying if we were writing raw SQL.
          // But Supabase JS client supports JSON filtering.
          const { data: users } = await supabase
            .from("preferences")
            .select("id, prefs_json")
            .filter("prefs_json->>stripe_customer_id", "eq", customerId);

          if (users && users.length > 0) {
            const user = users[0];
            const prefs = user.prefs_json || {};
            prefs.is_premium = false;

            await supabase
              .from("preferences")
              .upsert({ id: user.id, prefs_json: prefs });
          }
          break;
        }
        default:
          console.log(`Unhandled event type ${event.type}`);
      }

      return new Response(JSON.stringify({ received: true }), {
        headers: { "Content-Type": "application/json" },
      });
    } catch (err) {
      console.error("Webhook processing failed.", err);
      return new Response("Internal Server Error", { status: 500 });
    }
  },
};
